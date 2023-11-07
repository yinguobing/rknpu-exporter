use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use rocket::State;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate rocket;

// Define a type representing a metric label set.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct Labels {
    manufacturer: String,
    device: String,
    id: u32,
}

struct PromResources {
    registry: Registry,
    metrics: Family<Labels, Gauge>,
}

#[get("/metrics")]
fn index(prom: &State<PromResources>) -> String {
    // Read system NPU state
    let path = Path::new("/sys/kernel/debug/rknpu/load");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, s),
    }

    // Update metrics
    let npu_load = &prom.metrics;
    let npu_ids = vec![0, 1, 2];
    for id in npu_ids {
        npu_load
            .get_or_create(&Labels {
                manufacturer: "rockchip".to_string(),
                device: "npu".to_string(),
                id,
            })
            .set(42);
    }

    // Encode all metrics in the registry in the text format.
    let mut buffer = String::new();
    encode(&mut buffer, &prom.registry).unwrap();
    buffer
}

#[launch]
fn rocket() -> _ {
    // Create a metric registry.
    let mut registry = <Registry>::default();

    // Create a gauge metric family representing the npu load.
    let npu_load = Family::<Labels, Gauge>::default();

    // Register the metric family with the registry.
    registry.register("rockchip_npu_load", "Rockchip NPU Load", npu_load.clone());

    // Keep the Prometheus resources in Rocket state.
    let state = PromResources {
        registry,
        metrics: npu_load,
    };

    // Build the rocket.
    rocket::build().mount("/", routes![index]).manage(state)
}
