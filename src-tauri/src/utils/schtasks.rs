const CREATE_NO_WINDOW: u32 = 0x08000000;
const TASK_NAME_USER: &str = "Clash Verge";
const TASK_NAME_ADMIN: &str = "Clash Verge (Admin)";
const TASK_XML_DIR: &str = "tasks";
const TASK_XML_USER: &str = "clash-verge-task-user.xml";
const TASK_XML_ADMIN: &str = "clash-verge-task-admin.xml";

#[derive(Clone, Copy)]
pub enum TaskMode {
    User,
    Admin,
}

impl TaskMode {
    const fn name(self) -> &'static str {
        match self {
            Self::User => TASK_NAME_USER,
            Self::Admin => TASK_NAME_ADMIN,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
        }
    }
    const fn xml_run_level(self) -> &'static str {
        match self {
            Self::User => "LeastPrivilege",
            Self::Admin => "HighestAvailable",
        }
    }

    const fn xml_file_name(self) -> &'static str {
        match self {
            Self::User => TASK_XML_USER,
            Self::Admin => TASK_XML_ADMIN,
        }
    }
}

fn get_exe_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().map_err(|e| anyhow!("failed to get exe path:{}", e))?;
    Ok(exe_path)
}

fn get_task_user_id() -> Result<String> {
    let username = std::env::var_os("USERNAME")
        .or_else(|| std::env::var_os("USER"))
        .ok_or_else(|| anyhow!("failed to get current user name"))?;
    let username = username.to_string_lossy();
    let username = username.trim();
    if username.is_empty() {
        return Err(anyhow!("current user name is empty"));
    }

    let domain = std::env::var_os("USERDOMAIN")
        .or_else(|| std::env::var_os("COMPUTERNAME"))
        .map(|s| s.to_string_lossy().to_string());

    if let Some(domain) = domain {
        let domain = domain.trim();
        if !domain.is_empty() {
            return Ok(format!("{domain}\\{username}"));
        }
    }
    Ok(username.to_string())
}

fn get_startup_dir() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|e| anyhow!("failed to get appdata path: {}", e))?;
    let startup_dir = Path::new(&appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup");
    
    if !startup_dir.exists() {
        return Err(anyhow!("startup dir does not exist: {}", startup_dir.display()));
    }
    Ok(startup_dir.into())
}

async fn cleanup_legacy_task() -> Result<()> {
    let startup_dir = get_startup_dir()?;
    let old_shortcut = startup_dir.join("Clash-Verge.lnk");
    let new_shortcut = startup_dir.join("Clash Verge.lnk");

    old_shortcut.remove_if_exists().await?;
    new_shortcut.remove_if_exists().await?;
    Ok(())
}


fn task_xml_path(mode: TaskMode) -> Result<PathBuf> {
    let dir = dirs::app_home_dir()?.join(TASK_XML_DIR);
    fs::create_dir_all(&dir).map_err(|e| anyhow!("failed to create task xml dir: {}", e))?;
    Ok(dir.join(mode.xml_file_name()))
}