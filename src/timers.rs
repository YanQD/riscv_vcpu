extern crate alloc;

use alloc::boxed::Box;
use axhal::time::wall_time;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};
use riscv::register::sie;

const TICKS_PER_SEC: u64 = 100;
const NANOS_PER_SEC: u64 = 1_000_000_000;
const PERIODIC_INTERVAL_NANOS: u64 = NANOS_PER_SEC / TICKS_PER_SEC;

// TODO:complete TimerEventFn: including guest owmer, ...
pub struct TimerEventFn(Box<dyn FnOnce(TimeValue) + Send + 'static>);

impl TimerEventFn {
    /// Constructs a new [`TimerEventFn`] from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(TimeValue) + Send + 'static,
    {
        Self(Box::new(f))
    }
}

impl TimerEvent for TimerEventFn {
    fn callback(self, now: TimeValue) {
        (self.0)(now)
    }
}

#[percpu::def_percpu]
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<TimerEventFn>>> = LazyInit::new();

pub fn register_timer(deadline: usize, handler: TimerEventFn) {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    let mut timers = timer_list.lock();
    timers.set(TimeValue::from_nanos(deadline as u64), handler);
}

pub fn check_events() {
    loop {
        let now = wall_time();
        let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
        let event = timer_list.lock().expire_one(now);
        if let Some((_deadline, event)) = event {
            // info!("pick one to handler!!!");
            event.callback(now);
        } else {
            break;
        }
    }
}

pub fn scheduler_next_event() {
    // info!("set deadline!!!");
    let now_ns = axhal::time::monotonic_time_nanos();
    let deadline = now_ns + PERIODIC_INTERVAL_NANOS;
    axhal::time::set_oneshot_timer(deadline);
}

pub fn init() {
    let timer_list = unsafe { TIMER_LIST.current_ref_mut_raw() };
    timer_list.init_once(SpinNoIrq::new(TimerList::new()));

    axhal::irq::register_handler(axhal::time::TIMER_IRQ_NUM, || {
        check_events();
        scheduler_next_event();
    });
}