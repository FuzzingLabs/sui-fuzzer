use bichannel::Channel;

use crate::worker::worker::WorkerEvent;

const STATE_INIT_POSTFIX: &str = "init";

pub struct StatefulWorker {
    channel: Channel<WorkerEvent, WorkerEvent>,
}
