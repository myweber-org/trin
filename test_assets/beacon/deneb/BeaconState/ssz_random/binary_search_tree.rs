struct Node<T: Ord> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T: Ord> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

struct BinarySearchTree<T: Ord> {
    root: Option<Box<Node<T>>>,
}

impl<T: Ord> BinarySearchTree<T> {
    fn new() -> Self {
        BinarySearchTree { root: None }
    }

    fn insert(&mut self, value: T) {
        let new_node = Box::new(Node::new(value));
        self.root = Self::insert_node(self.root.take(), new_node);
    }

    fn insert_node(
        root: Option<Box<Node<T>>>,
        new_node: Box<Node<T>>,
    ) -> Option<Box<Node<T>>> {
        match root {
            None => Some(new_node),
            Some(mut node) => {
                if new_node.value < node.value {
                    node.left = Self::insert_node(node.left.take(), new_node);
                } else {
                    node.right = Self::insert_node(node.right.take(), new_node);
                }
                Some(node)
            }
        }
    }

    fn in_order_traversal(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::traverse(&self.root, &mut result);
        result
    }

    fn traverse<'a>(node: &'a Option<Box<Node<T>>>, result: &mut Vec<&'a T>) {
        if let Some(ref n) = node {
            Self::traverse(&n.left, result);
            result.push(&n.value);
            Self::traverse(&n.right, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bst_insert_and_traversal() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(2);
        bst.insert(4);
        bst.insert(6);
        bst.insert(8);

        let traversal_result = bst.in_order_traversal();
        let expected = vec![&2, &3, &4, &5, &6, &7, &8];
        assert_eq!(traversal_result, expected);
    }

    #[test]
    fn test_empty_bst() {
        let bst: BinarySearchTree<i32> = BinarySearchTree::new();
        assert!(bst.in_order_traversal().is_empty());
    }
}