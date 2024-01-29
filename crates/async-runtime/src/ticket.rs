//! Ticket is a type of Future that can wait for a result from Node.js.

use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    task::{Poll, Waker},
};

thread_local! {
    static TICKETS: RefCell<TicketRegistry> = RefCell::new(TicketRegistry::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TicketId(u32);

/// Issue a new string ticket.
pub fn issue_string_ticket() -> Ticket {
    TICKETS.with(|tickets| {
        let mut tickets = tickets.borrow_mut();
        tickets.issue()
    })
}

/// Receive a result from Node.js.
#[no_mangle]
pub extern "C" fn execute_node_ret(id: u32, result: *const u8, result_len: usize) {
    TICKETS.with(|tickets| {
        let mut tickets = tickets.borrow_mut();
        let id = TicketId(id);
        let result = unsafe {
            let slice = std::slice::from_raw_parts(result, result_len);
            String::from_utf8_unchecked(slice.to_vec())
        };
        let ticket = tickets
            .string_tickets
            .get_mut(&id)
            .expect("invalid ticket id");
        ticket.result = Some(result);
        ticket.waker.wake_by_ref();
    })
}

struct TicketRegistry {
    next_id: u32,
    string_tickets: HashMap<TicketId, StringTicket>,
}

impl TicketRegistry {
    fn new() -> Self {
        Self {
            next_id: 0,
            string_tickets: HashMap::new(),
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
        self.string_tickets.insert(
            id,
            StringTicket {
                waker,
                result: None,
            },
        );
        Ticket { id }
    }

    /// Takes the result of a ticket.
    fn take_result(&mut self, id: TicketId) -> Option<String> {
        let ticket = self.string_tickets.remove(&id).expect("invalid ticket id");
        match ticket.result {
            Some(result) => Some(result),
            None => {
                self.string_tickets.insert(id, ticket);
                None
            }
        }
    }
}

struct StringTicket {
    waker: Waker,
    result: Option<String>,
}

/// Ticket is a type of Future that can wait for a result from Node.js.
/// The id of a ticket should be passed to Node.js function.
/// When the result is available from Node.js, the ticket will be woken up.
/// This is done by calling `execute_node_ret` function from Node.js.
pub struct Ticket {
    pub id: TicketId,
}

impl Future for Ticket {
    type Output = String;

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
