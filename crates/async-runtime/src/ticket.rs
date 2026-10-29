//! Ticket is a type of Future that can wait for a result from Node.js.

use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    task::{Poll, Waker},
};

use crate::drive;

thread_local! {
    static TICKETS: RefCell<TicketRegistry> = RefCell::new(TicketRegistry::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TicketId(u32);

impl From<TicketId> for u32 {
    fn from(id: TicketId) -> Self {
        id.0
    }
}

/// Issue a new ticket.
pub fn issue_ticket() -> Ticket {
    TICKETS.with(|tickets| {
        let mut tickets = tickets.borrow_mut();
        tickets.issue()
    })
}

/// Receive a result from Node.js.
#[unsafe(no_mangle)]
pub extern "C" fn execute_node_ret(id: u32, is_ok: u32, result: *const u8, result_len: usize) {
    TICKETS.with(|tickets| {
        let mut tickets = tickets.borrow_mut();
        let id = TicketId(id);
        let ticket = tickets.tickets.get_mut(&id).expect("invalid ticket id");
        ticket.result = if is_ok != 0 {
            let result = unsafe { std::slice::from_raw_parts(result, result_len) };
            Some(Ok(result.to_vec()))
        } else {
            // result may be a null pointer on failure.
            Some(Err(()))
        };
        ticket.waker.wake_by_ref();
    });
    drive();
}

struct TicketRegistry {
    next_id: u32,
    tickets: HashMap<TicketId, PendingTicket>,
}

impl TicketRegistry {
    fn new() -> Self {
        Self {
            next_id: 0,
            tickets: HashMap::new(),
        }
    }

    /// Issues a new ticket.
    fn issue(&mut self) -> Ticket {
        let id = self.next_id;
        self.next_id += 1;
        let id = TicketId(id);
        Ticket { id }
    }

    /// Register a ticket which got a waker.
    fn register_ticket(&mut self, id: TicketId, waker: Waker) -> Ticket {
        self.tickets.insert(
            id,
            PendingTicket {
                waker,
                result: None,
            },
        );
        Ticket { id }
    }

    /// Takes the result of a ticket.
    fn take_result(&mut self, id: TicketId) -> Option<Result<Vec<u8>, ()>> {
        let ticket = self.tickets.remove(&id)?;
        match ticket.result {
            Some(result) => Some(result),
            None => {
                self.tickets.insert(id, ticket);
                None
            }
        }
    }
}

struct PendingTicket {
    waker: Waker,
    result: Option<Result<Vec<u8>, ()>>,
}

/// Ticket is a type of Future that can wait for a result from Node.js.
/// The id of a ticket should be passed to Node.js function.
/// When the result is available from Node.js, the ticket will be woken up.
/// The result is given as bytes, which are usually a UTF-8 string.
/// This is done by calling `execute_node_ret` function from Node.js.
pub struct Ticket {
    pub id: TicketId,
}

impl Future for Ticket {
    type Output = Result<Vec<u8>, ()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        TICKETS.with(|tickets| {
            let mut tickets = tickets.borrow_mut();
            if let Some(result) = tickets.take_result(self.id) {
                return Poll::Ready(result);
            }
            let waker = ctx.waker().clone();
            tickets.register_ticket(self.id, waker);
            Poll::Pending
        })
    }
}
