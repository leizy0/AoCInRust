use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::Error;

// Input port for process, data source
pub trait InputPort {
    fn get(&mut self) -> Option<i64>;
    fn reg_proc(&mut self, proc_id: usize);
}

// Output port for process, data sink
pub trait OutputPort {
    fn put(&mut self, value: i64) -> Result<(), Error>;
    fn wait_proc_id(&self) -> Option<usize>;
}

pub trait IOPort: InputPort + OutputPort {}
impl<T: InputPort + OutputPort> IOPort for T {}

pub trait Ref<T: ?Sized>: Clone {
    fn apply<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R;

    fn apply_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;
}

pub struct SeqRef<T: ?Sized> {
    r: Rc<RefCell<T>>,
}

impl<T> SeqRef<T> {
    fn new(t: T) -> Self {
        SeqRef {
            r: Rc::new(RefCell::new(t)),
        }
    }
}

pub type SeqInputRef = SeqRef<dyn InputPort>;
pub type SeqOutputRef = SeqRef<dyn OutputPort>;

impl<T: InputPort + 'static> SeqRef<T> {
    fn input_ref(&self) -> SeqInputRef {
        SeqRef { r: self.r.clone() }
    }
}

impl<T: OutputPort + 'static> SeqRef<T> {
    fn output_ref(&self) -> SeqOutputRef {
        SeqRef { r: self.r.clone() }
    }
}

impl<T: ?Sized> Clone for SeqRef<T> {
    fn clone(&self) -> Self {
        Self { r: self.r.clone() }
    }
}

impl<T: ?Sized> Ref<T> for SeqRef<T> {
    fn apply<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.r.borrow())
    }

    fn apply_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut *self.r.borrow_mut())
    }
}

pub struct ParaRef<T: ?Sized + Send> {
    r: Arc<Mutex<T>>,
}

impl<T: Send> ParaRef<T> {
    fn new(t: T) -> Self {
        ParaRef {
            r: Arc::new(Mutex::new(t)),
        }
    }
}

pub type ParaInputRef = ParaRef<dyn InputPort + Send>;
pub type ParaOutputRef = ParaRef<dyn OutputPort + Send>;

impl<T: InputPort + 'static + Send> ParaRef<T> {
    fn input_ref(&self) -> ParaInputRef {
        ParaRef { r: self.r.clone() }
    }
}

impl<T: OutputPort + 'static + Send> ParaRef<T> {
    fn output_ref(&self) -> ParaOutputRef {
        ParaRef { r: self.r.clone() }
    }
}

impl<T: ?Sized + Send> Clone for ParaRef<T> {
    fn clone(&self) -> Self {
        Self { r: self.r.clone() }
    }
}

impl<T: ?Sized + Send> Ref<T> for ParaRef<T> {
    fn apply<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.r.lock().unwrap())
    }

    fn apply_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.r.lock().unwrap())
    }
}

macro_rules! def_device {
    (name=$name:ident, ref_type=$ref_type:ident, input_ref_type=$input_ref_type:ident $(, device_additional_markers=$device_additional_markers:tt)?) => {
        // Input device
        pub struct $name<P: InputPort $(+ $device_additional_markers)? + 'static> {
            r: $ref_type<P>,
        }

        impl<P: InputPort $(+ $device_additional_markers)? + 'static> Clone for $name<P> {
            fn clone(&self) -> Self {
                Self {
                    r: self.r.clone(),
                }
            }
        }

        impl<P: InputPort $(+ $device_additional_markers)? + 'static> $name<P> {
            pub fn new(port: P) -> Self {
                Self {
                    r: $ref_type::new(port),
                }
            }

            pub fn check<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&P) -> U,
            {
                self.r.apply(f)
            }

            pub fn tweak<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&mut P) -> U,
            {
                self.r.apply_mut(f)
            }

            pub(super) fn input_port(&self) -> $input_ref_type {
                self.r.input_ref()
            }
        }
    };
    (name=$name:ident, ref_type=$ref_type:ident, output_ref_type=$output_ref_type:ident$(, device_additional_markers=$device_additional_markers:tt)?) => {
        // Output device
        pub struct $name<P: OutputPort $(+ $device_additional_markers)? + 'static> {
            r: $ref_type<P>,
        }

        impl<P: OutputPort $(+ $device_additional_markers)? + 'static> Clone for $name<P> {
            fn clone(&self) -> Self {
                Self {
                    r: self.r.clone(),
                }
            }
        }

        impl<P: OutputPort $(+ $device_additional_markers)? + 'static> $name<P> {
            pub fn new(port: P) -> Self {
                Self {
                    r: $ref_type::new(port),
                }
            }

            pub fn check<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&P) -> U,
            {
                self.r.apply(f)
            }

            pub fn tweak<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&mut P) -> U,
            {
                self.r.apply_mut(f)
            }

            pub(super) fn output_port(&self) -> $output_ref_type {
                self.r.output_ref()
            }
        }
    };
    (name=$name:ident, ref_type=$ref_type:ident, input_device_type=$input_device_type:ident, output_device_type=$output_device_type:ident $(, device_additional_markers=$device_additional_markers:tt)?) => {
        // I/O device
        pub struct $name<P: IOPort $(+ $device_additional_markers)? + 'static> {
            r: $ref_type<P>,
        }

        impl<P: IOPort $(+ $device_additional_markers)? + 'static> Clone for $name<P> {
            fn clone(&self) -> Self {
                Self {
                    r: self.r.clone(),
                }
            }
        }

        impl<P: IOPort $(+ $device_additional_markers)? + 'static> $name<P> {
            pub fn new(port: P) -> Self {
                Self {
                    r: $ref_type::new(port),
                }
            }

            pub fn check<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&P) -> U,
            {
                self.r.apply(f)
            }

            pub fn tweak<F, U>(&self, f: F) -> U
            where
                F: FnOnce(&mut P) -> U,
            {
                self.r.apply_mut(f)
            }

            pub fn input_device(&self) -> $input_device_type<P> {
                $input_device_type {
                    r: self.r.clone(),
                }
            }

            pub fn output_device(&self) -> $output_device_type<P> {
                $output_device_type {
                    r: self.r.clone()
                }
            }
        }
    };
}

def_device!(
    name = SeqInputDevice,
    ref_type = SeqRef,
    input_ref_type = SeqInputRef
);
def_device!(
    name = ParaInputDevice,
    ref_type = ParaRef,
    input_ref_type = ParaInputRef,
    device_additional_markers = Send
);
def_device!(
    name = SeqOutputDevice,
    ref_type = SeqRef,
    output_ref_type = SeqOutputRef
);
def_device!(
    name = ParaOutputDevice,
    ref_type = ParaRef,
    output_ref_type = ParaOutputRef,
    device_additional_markers = Send
);
def_device!(
    name = SeqIODevice,
    ref_type = SeqRef,
    input_device_type = SeqInputDevice,
    output_device_type = SeqOutputDevice
);
def_device!(
    name = ParaIODevice,
    ref_type = ParaRef,
    input_device_type = ParaInputDevice,
    output_device_type = ParaOutputDevice,
    device_additional_markers = Send
);

pub struct Channel {
    data: VecDeque<i64>,
    output_reg_proc_id: Option<usize>,
}

impl InputPort for Channel {
    fn get(&mut self) -> Option<i64> {
        self.data.pop_front()
    }

    fn reg_proc(&mut self, proc_id: usize) {
        self.output_reg_proc_id = Some(proc_id);
    }
}

impl OutputPort for Channel {
    fn put(&mut self, value: i64) -> Result<(), Error> {
        Ok(self.data.push_back(value))
    }

    fn wait_proc_id(&self) -> Option<usize> {
        self.output_reg_proc_id
    }
}

impl Channel {
    pub fn new(init_input: &[i64]) -> Self {
        Self {
            data: VecDeque::from_iter(init_input.iter().copied()),
            output_reg_proc_id: None,
        }
    }

    pub fn data(&self) -> &VecDeque<i64> {
        &self.data
    }
}
