use amd_smu_lib::{Codename, SmuReader};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

fn create_mock_sysfs() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Create mock sysfs files
    fs::write(path.join("version"), "SMU v46.54.0\n").unwrap();
    fs::write(path.join("drv_version"), "0.1.7\n").unwrap();
    fs::write(path.join("codename"), "12\n").unwrap(); // Vermeer
    fs::write(path.join("pm_table_version"), "0x240903\n").unwrap();
    fs::write(path.join("pm_table_size"), "6832\n").unwrap();

    // Create mock PM table
    let pm_table = create_mock_pm_table();
    let mut file = fs::File::create(path.join("pm_table")).unwrap();
    file.write_all(&pm_table).unwrap();

    dir
}

fn create_mock_pm_table() -> Vec<u8> {
    let mut data = vec![0u8; 6832];

    let write_f32 = |data: &mut [u8], offset: usize, value: f32| {
        let bytes = value.to_le_bytes();
        data[offset..offset + 4].copy_from_slice(&bytes);
    };

    // Limits and values (correct offsets from pm_table_0x240903 struct)
    write_f32(&mut data, 0x000, 142.0);  // PPT_LIMIT
    write_f32(&mut data, 0x004, 89.5);   // PPT_VALUE
    write_f32(&mut data, 0x008, 95.0);   // TDC_LIMIT
    write_f32(&mut data, 0x00C, 62.3);   // TDC_VALUE
    write_f32(&mut data, 0x010, 90.0);   // THM_LIMIT
    write_f32(&mut data, 0x014, 65.2);   // THM_VALUE (Tctl)
    write_f32(&mut data, 0x020, 140.0);  // EDC_LIMIT
    write_f32(&mut data, 0x024, 98.7);   // EDC_VALUE
    write_f32(&mut data, 0x060, 88.5);   // VDDCR_CPU_POWER
    write_f32(&mut data, 0x064, 12.4);   // VDDCR_SOC_POWER
    write_f32(&mut data, 0x0A0, 1.35);   // CPU_TELEMETRY_VOLTAGE
    write_f32(&mut data, 0x0B4, 1.10);   // SOC_TELEMETRY_VOLTAGE
    write_f32(&mut data, 0x0C0, 1800.0); // FCLK_FREQ
    write_f32(&mut data, 0x0CC, 1800.0); // MEMCLK_FREQ (was 0x0C8, now correct)
    write_f32(&mut data, 0x1CC, 42.1);   // SOC_TEMP (was 0x0A8, now correct)

    // Per-core data (8 cores) - correct offsets
    for i in 0..8 {
        write_f32(&mut data, 0x24C + i * 4, 8.0 + i as f32 * 0.5);   // CORE_POWER
        write_f32(&mut data, 0x28C + i * 4, 60.0 + i as f32 * 0.5);  // CORE_TEMP (was 0x2C0)
        write_f32(&mut data, 0x2EC + i * 4, 4500.0 + i as f32 * 50.0); // CORE_FREQ
        write_f32(&mut data, 0x30C + i * 4, 4400.0 + i as f32 * 50.0); // CORE_FREQEFF
        write_f32(&mut data, 0x32C + i * 4, 90.0 + i as f32);        // CORE_C0
    }

    data
}

#[test]
fn test_smu_reader_with_mock_sysfs() {
    let mock_dir = create_mock_sysfs();
    let reader = SmuReader::with_path(mock_dir.path()).unwrap();

    assert_eq!(reader.smu_version().unwrap().trim(), "SMU v46.54.0");
    assert_eq!(reader.driver_version().unwrap().trim(), "0.1.7");
    assert_eq!(reader.codename().unwrap(), Codename::Vermeer);
    assert_eq!(reader.pm_table_version().unwrap(), 0x240903);
}

#[test]
fn test_read_pm_table_with_mock() {
    let mock_dir = create_mock_sysfs();
    let reader = SmuReader::with_path(mock_dir.path()).unwrap();
    let table = reader.read_pm_table().unwrap();

    assert!((table.tctl - 65.2).abs() < 0.01);
    assert!((table.soc_temp - 42.1).abs() < 0.01);
    assert!((table.ppt_limit - 142.0).abs() < 0.01);
    assert!((table.fclk - 1800.0).abs() < 0.01);
}

#[test]
fn test_module_not_loaded() {
    let result = SmuReader::with_path("/nonexistent/path");
    assert!(result.is_err());
}
