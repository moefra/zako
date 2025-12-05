use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Once};

// =================================================================
// 1. 异步桥接层 (The Async Bridge)
// =================================================================

/// 定义一个可以在 V8 主线程执行的任务闭包
pub trait V8Task: Send {
    fn run(self: Box<Self>, scope: &mut ::v8::PinScope<'_, '_>);
}

impl<F> V8Task for F
where
    F: FnOnce(&mut v8::PinScope) + Send,
{
    fn run(self: Box<Self>, scope: &mut ::v8::PinScope<'_, '_>) {
        (*self)(scope)
    }
}

/// 发送给主循环的操作
pub type PendingOp = Box<dyn V8Task>;
