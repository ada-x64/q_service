use crate::prelude::*;
use bevy_ecs::{prelude::*, world::CommandQueue};
use bevy_tasks::{Task, futures_lite::future, prelude::*};
use tracing::{debug, warn};

/// A wrapper around a [bevy_tasks::Task] which can be returned
/// from the on_init or on_deinit hooks.
#[derive(Component)]
pub struct AsyncHook(pub Task<TaskResult>);

type TaskResult = Result<(), BevyError>;

// TODO: Trigger an event instead of polling every frame?
impl AsyncHook {
    /// Create an IO-bound task. Takes an async lambda as parameter. Uses the
    /// [IoTaskPool] as its backing executor. See those docs for usage info.
    ///
    /// ## Example usage
    /// ```
    /// fn my_init() -> InitResult {
    ///     let task = AsyncHook::io_task(async |q: CommandQueue| {
    ///         // ...
    ///     })
    ///     Ok(Some(task))
    /// }
    /// ```
    pub fn io_task(mut f: impl AsyncFnMut(CommandQueue) -> TaskResult + 'static) -> Self {
        let task = IoTaskPool::get().spawn_local(async move {
            let q = CommandQueue::default();
            (f)(q).await
        });
        AsyncHook(task)
    }
    /// Create an IO-bound task. Takes an async lambda as parameter. Uses the
    /// [ComputeTaskPool] as its backing executor. Note that this work must be
    /// completed to run the next frame.
    ///
    /// ## Example usage
    /// ```
    /// fn my_init() -> InitResult {
    ///     let task = AsyncHook::compute_task(async |q: CommandQueue| {
    ///         // ...
    ///     })
    ///     Ok(Some(task))
    /// }
    /// ```
    pub fn compute_task(mut f: impl AsyncFnMut(CommandQueue) -> TaskResult + 'static) -> Self {
        let task = ComputeTaskPool::get().spawn_local(async move {
            let q = CommandQueue::default();
            (f)(q).await
        });
        AsyncHook(task)
    }
    /// Create a compute-bound task with [AsyncComputeTaskPool] as its backing
    /// executor. Takes an async lambda as parameter. This work can span
    /// multiple frames.
    ///
    /// ## Example usage
    /// ```
    /// fn my_init() -> InitResult {
    ///     let task = AsyncHook::async_compute_task(async |q: CommandQueue| {
    ///         // ...
    ///     })
    ///     Ok(Some(task))
    /// }
    /// ```
    pub fn async_compute_task(
        mut f: impl AsyncFnMut(CommandQueue) -> TaskResult + 'static,
    ) -> Self {
        let task = AsyncComputeTaskPool::get().spawn_local(async move {
            let q = CommandQueue::default();
            (f)(q).await
        });
        AsyncHook(task)
    }
}

/// Poll tasks. This happens on PreUpdate.
pub(crate) fn poll_tasks<T: Service>(
    mut service: ServiceMut<T>,
    mut commands: Commands,
    mut q_tasks: Query<&mut AsyncHook>,
) {
    let tasks = std::mem::take(&mut service.tasks);
    let id = service.id();
    let status = service.status();
    if !status.is_initializing() && !status.is_deinitializing() && !tasks.is_empty() {
        warn!(
            "Non-empty task queue for service {} despite having status {status:?}",
            T::name()
        );
    }
    service.tasks = tasks
        .into_iter()
        .filter(|entity| {
            let mut task = q_tasks.get_mut(*entity).unwrap();
            let poll_res = block_on(future::poll_once(&mut task.0));
            let keep = poll_res.is_none();
            if let Some(res) = poll_res {
                match res {
                    Ok(_) => {
                        debug!("Finished task");
                        commands.entity(*entity).despawn();
                    }
                    Err(e) => commands.queue(move |world: &mut World| {
                        world.service_scope_by_id(id, |world, service| {
                            service.fail(world, ServiceError::Own(e.to_string()));
                        });
                    }),
                }
            }
            keep
        })
        .collect();
}
