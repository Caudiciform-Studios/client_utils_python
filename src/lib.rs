use pyo3::{ffi::c_str, prelude::*, types::{PySet, PyDict, PyBytes, PyTuple, PyList}};
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

use ::client_utils::{LocSet, LocSetIter, LocMap, bindings::{Loc, Command, ActionTarget, save_store, load_store, broadcast}, crdt::{CrdtContainer, ExpiringFWWRegister, ExpiringSet, SizedFWWExpiringSet, CrdtMap, Fww, Lww, Crdt}, framework::{ExplorableMap, State, Map}};

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

#[pyfunction]
pub fn convert<'py>(py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(r) = ::client_utils::behaviors::convert() {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
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
        ActionTarget::Items(items) => {
            let target_class = module.dict().get_item("ActionTarget_Items")?.unwrap();
            target_class.call1((items,))
        }
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

struct PyAnyCmp(Py<PyAny>);

impl std::clone::Clone for PyAnyCmp {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            PyAnyCmp(self.0.clone_ref(py))
        })
    }
}

impl std::cmp::Eq for PyAnyCmp {}

impl std::cmp::PartialEq for PyAnyCmp {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            self.0.bind(py).eq(other.0.bind(py)).unwrap_or(false)
        })
    }
}

impl std::cmp::Ord for PyAnyCmp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Python::with_gil(|py| {
            self.0.bind(py).compare(other.0.bind(py)).unwrap()
        })
    }
}

impl std::cmp::PartialOrd for PyAnyCmp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[pyclass(name="ExpiringFWWRegister")]
#[derive(Default)]
struct PyExpiringFWWRegister(ExpiringFWWRegister<PyAnyCmp>);

#[pymethods]
impl PyExpiringFWWRegister {
    pub fn get<'py>(&self, py: Python<'py>) -> Option<&Bound<'py, PyAny>> {
        self.0.get().map(|v| v.0.bind(py))
    }

    pub fn set<'py>(&mut self, now: i64, expires: i64, value: Bound<'py, PyAny>) {
        self.0.set(now, expires, PyAnyCmp(value.unbind()));
    }

    pub fn update_expiry<'py>(&mut self, py: Python<'py>, expires: i64) {
        self.0.update_expiry(expires);
    }

    pub fn merge<'py>(&mut self, py: Python<'py>, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }
}


#[pyclass(name="ExpiringSet")]
#[derive(Default)]
struct PyExpiringSet(ExpiringSet<PyAnyCmp>);

#[pymethods]
impl PyExpiringSet {
    pub fn insert<'py>(&mut self, value: Bound<'py, PyAny>, expires: i64) {
        self.0.insert(PyAnyCmp(value.unbind()), expires);
    }

    pub fn contains<'py>(&mut self, value: Bound<'py, PyAny>) -> bool {
        self.0.contains(&PyAnyCmp(value.unbind()))
    }

    pub fn merge<'py>(&mut self, py: Python<'py>, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }
}


#[pyclass(name="SizedFWWExpiringSet")]
struct PySizedFWWExpiringSet(SizedFWWExpiringSet<PyAnyCmp>);

#[pymethods]
impl PySizedFWWExpiringSet {
    #[new]
    pub fn new(size: usize) -> Self {
        Self(SizedFWWExpiringSet::new(size))
    }

    pub fn insert<'py>(&mut self, value: Bound<'py, PyAny>, now: i64, expires: i64) {
        self.0.insert(PyAnyCmp(value.unbind()), now, expires);
    }

    pub fn contains<'py>(&mut self, value: Bound<'py, PyAny>) -> bool {
        self.0.contains(&PyAnyCmp(value.unbind()))
    }

    pub fn merge<'py>(&mut self, py: Python<'py>, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }
}

#[pyclass(name="FwwCrdtMap")]
#[derive(Default)]
struct PyFwwCrdtMap(CrdtMap<PyAnyCmp, PyAnyCmp, Fww>);

#[pymethods]
impl PyFwwCrdtMap {
    pub fn insert<'py>(&mut self, key: Bound<'py, PyAny>, value: Bound<'py, PyAny>, now: i64) {
        self.0.insert(PyAnyCmp(key.unbind()), PyAnyCmp(value.unbind()), now);
    }

    pub fn contains_key<'py>(&mut self, key: Bound<'py, PyAny>) -> bool {
        self.0.contains_key(&PyAnyCmp(key.unbind()))
    }

    pub fn merge<'py>(&mut self, py: Python<'py>, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }
}

#[pyclass(name="LwwCrdtMap")]
#[derive(Default)]
struct PyLwwCrdtMap(CrdtMap<PyAnyCmp, PyAnyCmp, Lww>);

#[pymethods]
impl PyLwwCrdtMap {
    pub fn insert<'py>(&mut self, key: Bound<'py, PyAny>, value: Bound<'py, PyAny>, now: i64) {
        self.0.insert(PyAnyCmp(key.unbind()), PyAnyCmp(value.unbind()), now);
    }

    pub fn contains_key<'py>(&mut self, key: Bound<'py, PyAny>) -> bool {
        self.0.contains_key(&PyAnyCmp(key.unbind()))
    }

    pub fn merge<'py>(&mut self, py: Python<'py>, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }
}

#[pyclass(module="client_utils", name="ExplorableMap")]
#[derive(Default, Serialize, Deserialize)]
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

    pub fn nearest<'py>(&mut self, py: Python<'py>, tys: Vec<String>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if let Some(Loc { x, y }) = self.0.nearest(&tys) {
            let module = py.import_bound("client_utils")?;
            let loc_class = module.dict().get_item("Loc")?.unwrap();
            let loc = loc_class.call1((x, y))?;
            Ok(Some(loc))
        } else {
            Ok(None)
        }
    }

    pub fn move_towards<'py>(&mut self, py: Python<'py>, loc: Bound<'py, PyAny>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let x:i32 = loc.get_item("x")?.extract()?;
        let y:i32 = loc.get_item("y")?.extract()?;
        if let Some(r) = self.0.move_towards(Loc { x, y }) {
            Ok(Some(rust_command_to_py_command(r, py)?))
        } else {
            Ok(None)
        }
    }


    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyBytes::new(py, &bincode::serialize(&self.0).unwrap()).to_object(py))
    }

    pub fn __setstate__(&mut self, py: Python, state: Bound<PyBytes>) -> PyResult<()> {
        self.0 = bincode::deserialize(state.as_bytes()).unwrap();
        Ok(())
    }
}

#[pymodule(name = "client_utils")]
fn client_utils(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(astar, module)?)?;
    module.add_function(wrap_pyfunction!(convert, module)?)?;

    module.add_class::<PyExpiringFWWRegister>()?;
    module.add_class::<PyExpiringSet>()?;
    module.add_class::<PySizedFWWExpiringSet>()?;
    module.add_class::<PyFwwCrdtMap>()?;
    module.add_class::<PyLwwCrdtMap>()?;
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
