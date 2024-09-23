struct Node {
    data: i32,
    next_node: Link,
}

type Link = Option<Box<Node>>;

pub struct LinkedStack {
    head: Link,
}

impl LinkedStack {
    fn new() -> Self {
        LinkedStack { head: None }
    }

    fn push(&mut self, val: i32) {
        let new = Node {
            data: val,
            next_node: self.head.take(),
        };
        self.head = Some(Box::new(new));
    }

    fn pop(&mut self) -> Option<i32> {
        match self.head.take() {
            Some(node) => {
                self.head = node.next_node;
                Some(node.data)
            }
            None => None,
        }
    }
}

impl Drop for LinkedStack {
    fn drop(&mut self) {}
}

// DO NOT MODIFY BELOW THIS LINE

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack() {
        let mut stack = LinkedStack::new();
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_linked_stack() {
        let mut stack = LinkedStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));

        stack.push(4);

        assert_eq!(stack.pop(), Some(4));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_big_stack() {
        let mut stack = LinkedStack::new();
        for i in 0..1_000_000 {
            stack.push(i);
        }

        for i in (0..1_000_000).rev() {
            assert_eq!(stack.pop(), Some(i));
        }

        assert_eq!(stack.pop(), None);
    }
}
