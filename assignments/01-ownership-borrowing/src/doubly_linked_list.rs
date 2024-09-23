// Now try and implement a doubly linked version. Give an explanation
// for why this doesn't work.

struct Node {
    data: i32,
    nextNode: Link,
    pastNode: Link,
}

type Link = Option<Box<Node>>;

pub struct LinkedStack {
    head: Link,
    tail: Link,
}

impl LinkedStack {
    fn new() -> Self {
        let linkedstack = LinkedStack {
            head: None,
            tail: None,
        };
        linkedstack
    }

    fn push(&mut self, val: i32) {
        let mut new = Node {
            data: val,
            nextNode: self.head.take(),
            pastNode: None,
        };
        *new.nextNode.take();
        self.head = Some(Box::new(new));
    }

    fn pop(&mut self) -> Option<i32> {
        match self.head.take() {
            Some(Node) => {
                self.head = Node.nextNode;
                self.tail = Node.pastNode;
                Some(Node.data)
            }
            None => None,
        }
    }
}
