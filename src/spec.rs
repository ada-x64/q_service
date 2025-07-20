use crate::prelude::*;
use bevy_platform::prelude::*;

#[derive(Debug)]
pub(crate) struct ServiceSpec<T: Service> {
    pub deps: Vec<NodeId>,
    pub on_init: Option<InitHook<T>>,
    pub on_deinit: Option<DeinitHook<T>>,
    pub on_up: Option<UpHook<T>>,
    pub on_down: Option<DownHook<T>>,
    pub is_startup: bool,
}

impl<T> Default for ServiceSpec<T>
where
    T: Service,
{
    fn default() -> Self {
        Self {
            deps: vec![],
            on_init: None,
            on_deinit: None,
            on_up: None,
            on_down: None,
            is_startup: false,
        }
    }
}
