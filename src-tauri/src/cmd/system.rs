use anyhow::Result;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

/// Get system hostname
#[tauri::command]
pub fn get_system_hostname() -> String {
    gethostname::gethostname().to_string_lossy().to_string()
}

#[tauri::command]
pub fn get_network_interface() -> Result<serde_json::Value, String> {
    let interfaces = NetworkInterface::show().map_err(|e| e.to_string())?;
    let interface_json = interfaces
        .into_iter()
        .map(|iface| {
            serde_json::json!({
                "name": iface.name,
                "ip_count": iface.addr.len(),
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!(interface_json))
}

/// Check if port is in use
#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    let hostname = get_system_hostname();
    let interface_json = get_network_interface()?;
    Ok(serde_json::json!({
        "hostname": hostname,
        "interface": interface_json,
    }))
}




