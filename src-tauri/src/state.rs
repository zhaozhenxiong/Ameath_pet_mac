use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PetState {
    Idle,
    Thinking,
    Running,
    Review,
    Failed,
    Waving,
    Jumping,
    Chatting,
    Fetching,
    Searching,
    Analyzing,
    Building,
    Celebrating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRecord {
    pub state: PetState,
    pub tool: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateChangeEvent {
    pub animation: String,
    pub bubble: String,
}

pub type SharedState = Arc<Mutex<StateManager>>;

pub struct StateManager {
    current: PetState,
    current_tool: Option<String>,
    current_bubble: Option<String>,
    current_provider: String,
    history: Vec<StateRecord>,
    pub last_transition: u64,
    permission_lock_until: u64,
}

impl PetState {
    pub fn from_hook(s: &str, _tool: Option<&str>) -> Self {
        match s {
            "thinking" => PetState::Thinking,
            "working" => PetState::Running,
            "done" => PetState::Review,
            "idle" => PetState::Idle,
            "error" => PetState::Failed,
            "jumping" => PetState::Jumping,
            "waving" => PetState::Waving,
            "chatting" => PetState::Chatting,
            "fetching" => PetState::Fetching,
            "searching" => PetState::Searching,
            "analyzing" => PetState::Analyzing,
            "building" => PetState::Building,
            "celebrating" => PetState::Celebrating,
            _ => PetState::Idle,
        }
    }

    pub fn animation_name(&self) -> &'static str {
        match self {
            PetState::Idle => "idle",
            PetState::Thinking => "waiting",
            PetState::Running => "running",
            PetState::Review => "review",
            PetState::Failed => "failed",
            PetState::Waving => "waving",
            PetState::Jumping => "jumping",
            PetState::Chatting => "chatting",
            PetState::Fetching => "fetching",
            PetState::Searching => "searching",
            PetState::Analyzing => "analyzing",
            PetState::Building => "building",
            PetState::Celebrating => "celebrating",
        }
    }

    pub fn bubble_text(&self, tool: Option<&str>) -> &'static str {
        match self {
            PetState::Thinking => "正在思考...",
            PetState::Running => match tool {
                Some("Read") | Some("Glob") | Some("Grep") | Some("read_file") => "正在读取文件...",
                Some("Write") | Some("Edit") | Some("apply_patch") => "正在写代码...",
                Some("Bash") | Some("exec_command") | Some("functions.exec") => "正在执行命令...",
                Some("Agent") | Some("Task") | Some("collaboration.spawn_agent") => {
                    "正在调度子任务..."
                }
                Some("WebFetch") | Some("web_fetch") => "正在获取网络内容...",
                Some("WebSearch") | Some("web.run") => "正在搜索网络...",
                _ => "工作中...",
            },
            PetState::Review => "搞定!",
            PetState::Failed => "好像出问题了...",
            PetState::Waving => "等待指示...",
            PetState::Jumping => "",
            PetState::Idle => "",
            PetState::Chatting => "正在组织回复...",
            PetState::Fetching => "正在获取网络内容...",
            PetState::Searching => "正在搜索网络...",
            PetState::Analyzing => "正在分析...",
            PetState::Building => "正在构建...",
            PetState::Celebrating => "太棒了!",
        }
    }
}

impl StateManager {
    pub fn new() -> Self {
        StateManager {
            current: PetState::Idle,
            current_tool: None,
            current_bubble: None,
            current_provider: "codex".to_string(),
            history: Vec::new(),
            last_transition: 0,
            permission_lock_until: 0,
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn set_state_from(&mut self, state: PetState, tool: Option<String>, provider: &str) {
        self.last_transition = Self::now_ms();
        self.history.push(StateRecord {
            state: state.clone(),
            tool: tool.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.current = state;
        self.current_tool = tool;
        self.current_bubble = None;
        self.current_provider = provider.to_string();
    }

    pub fn set_custom_state(&mut self, state: PetState, bubble: Option<String>, provider: &str) {
        self.set_state_from(state, None, provider);
        self.current_bubble = bubble;
    }

    pub fn is_permission_locked(&self) -> bool {
        Self::now_ms() < self.permission_lock_until
    }

    pub fn lock_permission(&mut self) {
        self.permission_lock_until = Self::now_ms() + 1500;
    }

    pub fn current_state(&self) -> &PetState {
        &self.current
    }

    pub fn current_tool(&self) -> Option<&str> {
        self.current_tool.as_deref()
    }

    pub fn current_bubble(&self) -> Option<&str> {
        self.current_bubble.as_deref()
    }

    pub fn current_provider(&self) -> &str {
        &self.current_provider
    }

    pub fn history(&self) -> &Vec<StateRecord> {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_from_hook() {
        assert_eq!(PetState::from_hook("thinking", None), PetState::Thinking);
        assert_eq!(
            PetState::from_hook("working", Some("Bash")),
            PetState::Running
        );
        assert_eq!(PetState::from_hook("done", None), PetState::Review);
        assert_eq!(PetState::from_hook("idle", None), PetState::Idle);
        assert_eq!(PetState::from_hook("error", None), PetState::Failed);
        assert_eq!(PetState::from_hook("unknown", None), PetState::Idle);
    }

    #[test]
    fn test_bubble_text_for_tools() {
        assert_eq!(
            PetState::Running.bubble_text(Some("Read")),
            "正在读取文件..."
        );
        assert_eq!(
            PetState::Running.bubble_text(Some("Write")),
            "正在写代码..."
        );
        assert_eq!(PetState::Running.bubble_text(Some("Edit")), "正在写代码...");
        assert_eq!(
            PetState::Running.bubble_text(Some("Bash")),
            "正在执行命令..."
        );
        assert_eq!(
            PetState::Running.bubble_text(Some("Agent")),
            "正在调度子任务..."
        );
        assert_eq!(
            PetState::Running.bubble_text(Some("WebFetch")),
            "正在获取网络内容..."
        );
    }

    #[test]
    fn test_bubble_text_for_states() {
        assert_eq!(PetState::Thinking.bubble_text(None), "正在思考...");
        assert_eq!(PetState::Review.bubble_text(None), "搞定!");
        assert_eq!(PetState::Failed.bubble_text(None), "好像出问题了...");
        assert_eq!(PetState::Waving.bubble_text(None), "等待指示...");
        assert_eq!(PetState::Idle.bubble_text(None), "");
    }

    #[test]
    fn test_state_manager_history() {
        let mut sm = StateManager::new();
        sm.set_state_from(PetState::Thinking, None, "codex");
        sm.set_state_from(PetState::Running, Some("Bash".into()), "claude");
        sm.set_state_from(PetState::Review, None, "hermes");
        assert_eq!(sm.history().len(), 3);
        assert_eq!(sm.current_state(), &PetState::Review);
        assert_eq!(sm.current_provider(), "hermes");
    }

    #[test]
    fn waving_only_locks_when_permission_is_requested() {
        let mut sm = StateManager::new();
        sm.set_state_from(PetState::Waving, None, "codex");
        assert!(!sm.is_permission_locked());
        sm.lock_permission();
        assert!(sm.is_permission_locked());
    }

    #[test]
    fn custom_bubble_is_replaced_by_the_next_lifecycle_state() {
        let mut sm = StateManager::new();
        sm.set_custom_state(PetState::Waving, Some("请选择".into()), "mcp");
        assert_eq!(sm.current_bubble(), Some("请选择"));
        sm.set_state_from(PetState::Running, Some("Bash".into()), "codex");
        assert_eq!(sm.current_bubble(), None);
    }

    #[test]
    fn test_animation_name_mapping() {
        assert_eq!(PetState::Idle.animation_name(), "idle");
        assert_eq!(PetState::Thinking.animation_name(), "waiting");
        assert_eq!(PetState::Running.animation_name(), "running");
        assert_eq!(PetState::Review.animation_name(), "review");
        assert_eq!(PetState::Failed.animation_name(), "failed");
        assert_eq!(PetState::Chatting.animation_name(), "chatting");
        assert_eq!(PetState::Fetching.animation_name(), "fetching");
        assert_eq!(PetState::Searching.animation_name(), "searching");
        assert_eq!(PetState::Analyzing.animation_name(), "analyzing");
        assert_eq!(PetState::Building.animation_name(), "building");
        assert_eq!(PetState::Celebrating.animation_name(), "celebrating");
    }

    #[test]
    fn test_new_state_bubbles() {
        assert_eq!(PetState::Chatting.bubble_text(None), "正在组织回复...");
        assert_eq!(PetState::Fetching.bubble_text(None), "正在获取网络内容...");
        assert_eq!(PetState::Searching.bubble_text(None), "正在搜索网络...");
        assert_eq!(PetState::Analyzing.bubble_text(None), "正在分析...");
        assert_eq!(PetState::Building.bubble_text(None), "正在构建...");
        assert_eq!(PetState::Celebrating.bubble_text(None), "太棒了!");
    }
}
