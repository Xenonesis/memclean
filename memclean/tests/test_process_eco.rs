use memclean::core::process::set_process_eco_qos;

#[test]
fn test_set_current_process_eco_qos() {
    let cur_pid = std::process::id();
    let res_enable = set_process_eco_qos(cur_pid, true);
    if let Ok(()) = res_enable {
        let res_disable = set_process_eco_qos(cur_pid, false);
        assert!(res_disable.is_ok(), "Disabling EcoQoS on current process should succeed");
    } else {
        // If unsupported or blocked by environment, it should return an informative error
        let err = res_enable.unwrap_err();
        assert!(!err.is_empty());
    }
}
