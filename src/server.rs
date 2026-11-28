use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{Implementation, ServerCapabilities, ServerConfig},
    tool_handler,
};

use crate::config::Inventory;
use crate::state::ConnectionPool;
use crate::transport::SessionManager;

/// The MCP server struct. Must be Clone — every field is Arc-backed.
///
/// Two session registries coexist:
/// - [`pool`](Self::pool): exec-per-channel SSH connections for the server-ops
///   tools (`ssh_connect`/`ssh_exec`, all `vps_*`/`ubuntu_*` tools), which
///   capture exit codes and run SFTP.
/// - [`sessions`](Self::sessions): persistent interactive console sessions
///   (SSH PTY / Telnet / Serial) for the switch/console tools (`connect`,
///   `run_command`, `enable`, …).
#[derive(Clone)]
pub struct SshConnectServer {
    pub pool: ConnectionPool,
    pub sessions: SessionManager,
    pub inventory: Arc<Inventory>,
    pub(crate) tool_router: ToolRouter<SshConnectServer>,
}

impl SshConnectServer {
    fn build_router() -> ToolRouter<SshConnectServer> {
        Self::ssh_tool_router()
            + Self::ubuntu_tool_router()
            + Self::system_tool_router()
            + Self::file_tool_router()
            + Self::service_tool_router()
            + Self::database_tool_router()
            + Self::extras_tool_router()
            + Self::health_tool_router()
            + Self::session_tool_router()
            + Self::interactive_tool_router()
            + Self::switch_tool_router()
    }

    /// Build a server with fresh local state.
    pub fn new() -> Self {
        Self {
            pool: ConnectionPool::new(),
            sessions: SessionManager::new(),
            inventory: Arc::new(Inventory::load_default()),
            tool_router: Self::build_router(),
        }
    }
}

/// Wire the composed tool_router into the MCP ServerHandler.
/// The macro generates call_tool(), list_tools(), and get_tool() using self.tool_router.
/// We provide get_info() manually so the macro skips auto-generation.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for SshConnectServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::new(
            "ssh-connect",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "SSH-Connect: unified remote management over SSH, Telnet, and Serial. \
             Two interaction styles share one session registry: \
             (1) Server ops — ssh_connect, then ssh_exec (exit codes), ssh_upload_file/ssh_download_file/ssh_list_files (SFTP), \
             vps_system_stats, vps_logs, vps_process, vps_file_read, vps_file_ops, vps_cron, \
             ubuntu_service_control, vps_nginx_config, ubuntu_nginx_control, ubuntu_update_packages, ubuntu_ssl_certificate, \
             ubuntu_website_deployment, ubuntu_ufw_firewall, ubuntu_mysql, vps_redis, vps_docker; \
             vps_health_check (one-call structured health battery — pair with the vps-health-report skill). \
             (2) Interactive console / network switches — connect (ssh|telnet|serial), then run_command/run_commands, \
             enable, login, expect_send, run_on_fleet; list_com_ports, list_hosts, switch_backup_config, switch_network_diagnostics. \
             Call ssh_disconnect / disconnect when done."
                .to_string(),
        )
    }
}
