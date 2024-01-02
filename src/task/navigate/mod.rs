use std::rc::Rc;

use crate::{controller::{Controller, self}, vision::matcher::MatchType};

mod page;
use super::{MatchTask, Task, MultipleMatchTask};

pub struct NavigateTask {

}