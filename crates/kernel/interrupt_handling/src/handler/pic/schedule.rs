use crate::handler::pic::get_duration;
use alloc::{boxed::Box, vec::Vec};
use core::time::Duration;
use kernel_spin::KernelMutex;

struct Task {
    due: Duration,
    function: Box<dyn 'static + Send + FnOnce()>,
}

type Tasks = Vec<Task>;

static TASKS: KernelMutex<Tasks> = KernelMutex::new(Tasks::new());

fn tasks<F, R>(f: F) -> R
where
    F: FnOnce(&mut Tasks) -> R,
{
    TASKS.lock(f)
}

fn add_task<F>(due: Duration, function: F)
where
    F: 'static + Send + FnOnce(),
{
    tasks(move |tasks| {
        tasks.push(Task {
            due,
            function: Box::new(function),
        });
        tasks.sort_unstable_by_key(|task| task.due);
    })
}

/// Schedules a task to be run at least this far in the future
///
/// Be careful, you are run in an interrupt context. Be a good neighbor
pub fn schedule_task<F>(duration: Duration, function: F)
where
    F: 'static + Send + FnOnce(),
{
    add_task(get_duration() + duration, function)
}

/// Run all tasks that have an expired timer.
///
/// This is to be called in the pic timer handler.
pub(crate) fn pump_tasks() {
    let current_time = get_duration();

    loop {
        match tasks(|tasks| {
            if tasks
                .first()
                .map(|task| task.due <= current_time)
                .unwrap_or(false)
            {
                Some(tasks.remove(0))
            } else {
                None
            }
        }) {
            Some(task) => (task.function)(),
            None => break,
        }
    }
}
