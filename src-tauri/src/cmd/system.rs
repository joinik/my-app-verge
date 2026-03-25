use serde::{Deserialize, Serialize};
use sysinfo::{Networks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_count: u32,
    pub used_memory: u64,
    pub total_memory: u64,
}

/// 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all(); // 确保所有信息都已刷新
    Ok(SystemInfo {
        os_name: System::name().unwrap_or_else(|| "Unknown".into()), // 修改为 System::name()
        os_version: System::os_version().unwrap_or_else(|| "Unknown".into()), // 修改为 System::os_version()
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".into()), // 修改为 System::kernel_version()
        cpu_count: sys.cpus().len() as u32,
        used_memory: sys.used_memory(),
        total_memory: sys.total_memory(),
    })
}

/// 获取系统主机名
#[tauri::command]
pub fn get_system_hostname() -> Result<String, String> {
    // 刷新所有信息以确保主机名是最新的，虽然 System::host_name() 是关联函数
    // 但为了确保数据一致性，仍然保留 refresh_all()
    let mut sys = System::new_all();
    sys.refresh_all();
    System::host_name() // 修改为 System::host_name()
        .ok_or_else(|| "Failed to get hostname".to_string())
}

/// 获取网络接口信息
#[tauri::command]
pub fn get_network_interface() -> Result<String, String> {
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();
    for (interface_name, network) in &networks {
        interfaces.push(format!(
            "Interface: {}, Total Received: {} bytes, Total Transmitted: {} bytes",
            interface_name,
            network.total_received(),
            network.total_transmitted()
        ));
    }
    if interfaces.is_empty() {
        Ok("No network interfaces found".to_string())
    } else {
        Ok(interfaces.join("\\n"))
    }
}
