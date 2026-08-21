use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let vec = vec![1, 2, 3];
    let shared_vec = Rc::new(RefCell::new(vec));
    let shared_vec_2 = Rc::clone(&shared_vec);
    shared_vec_2.borrow_mut().push(4);
    println!("{:?}", shared_vec.borrow());
}
