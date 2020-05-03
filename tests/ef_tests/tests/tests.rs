use ef_tests::*;
use test_env_log::test;

#[test]
fn vm_arithmetic_tests() {
    VmArithmeticHandler::run();
}

#[test]
fn vm_bitwise_operations_tests() {
    VmBitwiseOperationsHandler::run();
}

#[test]
fn vm_block_info_tests() {
    VmBlockInfoHandler::run();
}

#[test]
fn vm_environmental_info_tests() {
    VmEnvironmentalInfoHandler::run();
}

#[test]
fn vm_io_and_flow_tests() {
    VmIoAndFlowHandler::run();
}

#[ignore]
#[test]
fn vm_log_tests() {
    VmLogHandler::run();
}

#[ignore]
#[test]
fn vm_performance_tests() {
    VmPerformanceHandler::run();
}

#[test]
fn vm_push_dup_swap_tests() {
    VmPushDupSwapHandler::run();
}

#[test]
fn vm_random_tests() {
    VmRandomHandler::run();
}

#[test]
fn vm_sha3_tests() {
    VmSha3Handler::run();
}

#[test]
fn vm_system_operations_tests() {
    VmSystemOperationsHandler::run();
}

#[test]
fn vm_tests() {
    VmTestsHandler::run();
}
