use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::marker::PhantomData;
use tracing::debug;

#[derive(Event, Debug)]
pub(crate) enum LifecycleCommand<S: Service> {
    SpinUp,
    SpinDown,
    Restart,
    Fail(ServiceError),
    _Placeholder(PhantomData<S>),
}
impl<S: Service> LifecycleCommand<S> {
    /// Lower number = higher priority, should execute first.
    pub(crate) fn priority(&self, service_status: ServiceStatus) -> u8 {
        match self {
            LifecycleCommand::Fail(_) => 0,
            LifecycleCommand::Restart => 1,
            LifecycleCommand::SpinUp => {
                if service_status.is_up() {
                    3
                } else {
                    2
                }
            }
            LifecycleCommand::SpinDown => {
                if service_status.is_down() {
                    3
                } else {
                    2
                }
            }
            LifecycleCommand::_Placeholder(_) => unreachable!(),
        }
    }
}

/// Extensions for Commands to allow moving along the service lifecycle.
pub trait ServiceCommandsExt {
    /// Queue the service to be spun up. Will warn and do nothing if the service is already up.
    fn spin_service_up<S: Service>(&mut self);
    /// Queue the service to be spun down. Will warn and do nothing if the service is already down.
    fn spin_service_down<S: Service>(&mut self);
    /// Queue the service to be spun up, forcibly.
    fn restart_service<S: Service>(&mut self);
    /// Queues the service to fail with the given error. Will forcibly spin down the service.
    fn fail_service<S: Service>(&mut self, reason: ServiceError);
}
impl<'w, 's> ServiceCommandsExt for Commands<'w, 's> {
    fn spin_service_up<S: Service>(&mut self) {
        debug!("spin_service_up");
        self.send_event(LifecycleCommand::SpinUp::<S>);
    }

    fn spin_service_down<S: Service>(&mut self) {
        debug!("spin_service_up");
        self.send_event(LifecycleCommand::SpinDown::<S>);
    }

    fn restart_service<S: Service>(&mut self) {
        debug!("spin_service_up");
        self.send_event(LifecycleCommand::Restart::<S>);
    }

    fn fail_service<S: Service>(&mut self, reason: ServiceError) {
        debug!("spin_service_up");
        self.send_event(LifecycleCommand::Fail::<S>(reason));
    }
}

/// Executes any queued up service lifecycle commands.
#[tracing::instrument(skip_all)]
pub(crate) fn watch_service_commands<S: Service>(
    mut reader: EventReader<LifecycleCommand<S>>,
    mut commands: Commands,
    service: ServiceRef<S>,
) {
    let status = service.status();
    if let Some(event) = reader.read().min_by(|a, b| {
        let order = a.priority(status.clone()).cmp(&b.priority(status.clone()));
        debug!("{a:?}.cmp({b:?}) = {order:?}");
        order
    }) {
        debug!("({}) Got event {:?}", S::name(), event);
        match event {
            LifecycleCommand::SpinUp => commands.queue(|world: &mut World| {
                world.service_scope::<S, ()>(|world, service| service.spin_up(world));
            }),
            LifecycleCommand::SpinDown => commands.queue(|world: &mut World| {
                world.service_scope::<S, ()>(|world, service| service.spin_down(world));
            }),
            LifecycleCommand::Restart => commands.queue(|world: &mut World| {
                world.service_scope::<S, ()>(|world, service| service.restart(world));
            }),
            LifecycleCommand::Fail(error) => {
                let error = error.clone();
                commands.queue(move |world: &mut World| {
                    world.service_scope::<S, ()>(|world, service| {
                        service.fail(world, error.clone())
                    });
                })
            }
            _ => unreachable!(),
        }
    }
}
