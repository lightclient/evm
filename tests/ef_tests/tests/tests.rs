use ef_tests::{Handler, VmHandler};

#[test]
fn vm_tests() {
    // VmHandler::run(Some(vec!["mulUnderFlow.json"]));
    VmHandler::run(None);
}
