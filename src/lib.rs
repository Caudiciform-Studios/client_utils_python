use pyo3::{ffi::c_str, prelude::*, types::{PySet, PyDict, PyTuple, PyList}};
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

use ::client_utils::{LocSet, LocSetIter, LocMap, bindings::{Loc, Command, ActionTarget}, crdt::CrdtContainer, framework::{ExplorableMap, State, Map}};

struct PyLocSet<'a>(Bound<'a, PySet>);
impl <'a> LocSet for PyLocSet<'a> {
    fn contains_loc(&self, loc: &Loc) -> bool {
        self.0.contains((loc.x, loc.y)).unwrap_or(false)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter(&self) -> LocSetIter {
        LocSetIter { inner: Box::new(self.0.iter().map(|k| {
            let (x,y) = k.extract().unwrap();
            Loc {x, y}
        })) }
    }
}

struct PyLocMap<'a>(Bound<'a, PyDict>);
impl <'a>LocSet for PyLocMap<'a> {
    fn contains_loc(&self, loc: &Loc) -> bool {
        self.0.contains((loc.x, loc.y)).unwrap_or(false)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter(&self) -> LocSetIter {
        LocSetIter { inner: Box::new(self.0.iter().map(|(k,_v)| {
            let (x,y) = k.extract().unwrap();
            Loc {x, y}
        })) }
    }
}
impl <'a>LocMap for PyLocMap<'a> {
    fn get_loc(&self, loc: &Loc) -> Option<bool> {
        self.0.get_item((loc.x, loc.y)).unwrap().map(|v| v.is_truthy().unwrap() )
    }
}

#[pyfunction]
#[pyo3(pass_module)]
fn astar<'py>(m: &Bound<'py, PyModule>, current_loc: Bound<'py, PyTuple>, goal: Bound<'py, PyTuple>, explored_tiles: Bound<'py, PyDict>, blocked: Bound<'py, PySet>, avoid: Bound<'py, PySet>) -> PyResult<Option<Bound<'py, PyList>>> {
    let x = current_loc.get_item(0)?.extract::<i32>()?;
    let y = current_loc.get_item(1)?.extract::<i32>()?;
    let current_loc = Loc {x, y};
    let x = goal.get_item(0)?.extract::<i32>()?;
    let y = goal.get_item(1)?.extract::<i32>()?;
    let goal = Loc {x, y};
    let path = ::client_utils::astar(current_loc, goal, &PyLocMap(explored_tiles), &PyLocSet(blocked), &PyLocSet(avoid));
    let path = if let Some(path) = path {
        Some(PyList::new_bound(m.py(), path.iter().map(|l| (l.x, l.y))))
    } else {
        None
    };
    Ok(path)
}

fn rust_action_target_to_py_action_target<'py>(target: ActionTarget, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import_bound("client_utils")?;
    match target {
        ActionTarget::Location(loc) => {
            let target_class = module.dict().get_item("ActionTarget_Location")?.unwrap();
            let loc_class = module.dict().get_item("Loc")?.unwrap();
            let loc = loc_class.call1((loc.x, loc.y))?;
            target_class.call1((loc,))
        },
        _ => unimplemented!()
    }
}

fn rust_command_to_py_command<'py>(command: Command, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import_bound("client_utils")?;
    match command {
        Command::UseAction((index, target)) => {
            let command_class = module.dict().get_item("Command_UseAction")?.unwrap();
            let target = target.map(|target| rust_action_target_to_py_action_target(target, py).unwrap());
            command_class.call1(((index, target),))
        }
        Command::Nothing => {
            let command_class = module.dict().get_item("Command_Nothing")?.unwrap();
            command_class.call0()
        }
    }
}

#[pyclass(name="ExplorableMap")]
#[derive(Default)]
struct PyExplorableMap(ExplorableMap);

#[pymethods]
impl PyExplorableMap {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.0.update();
    }

    pub fn explore<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Some(r) = self.0.explore() {
            Ok(Some(rust_command_to_py_command(r, py)?))
        } else {
            Ok(None)
        }
    }

    pub fn move_towards_nearest<'py>(&mut self, py: Python<'py>, tys: Vec<String>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Some(r) = self.0.move_towards_nearest(&tys) {
            Ok(Some(rust_command_to_py_command(r, py)?))
        } else {
            Ok(None)
        }
    }
}

#[pymodule(name = "client_utils")]
fn client_utils(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(astar, module)?)?;
    module.add_class::<PyExplorableMap>()?;

    let code = include_str!("defs.py");
    let defs = PyModule::from_code_bound(module.py(), code, "defs.py", "defs")?;
    for (k,v) in defs.dict() {
        if let Ok(k) = k.extract::<&str>() {
            if !k.starts_with("__") {
                module.add(k, v)?;
            }
        }
    }

    Ok(())
}
