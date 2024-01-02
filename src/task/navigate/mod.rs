use std::rc::Rc;

use crate::controller::{Controller, self};

mod page;
use self::page::Page;

use super::{MatchTask, Exec, AndTask};

// pub struct NavigateTask {
//     target: Box<dyn Page>
// }