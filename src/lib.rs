use std::ffi::CString;
use pyo3::{prelude::*, types::{PySet, PyDict, PyBytes, PyTuple, PyList}, exceptions::PyTypeError};
use serde::{Serialize, Deserialize};

use ::client_utils::{
    LocSet, LocSetIter, LocMap,
    bindings::{Loc, Command, ActionTarget, EquipmentSlot, Direction},
    crdt::{GrowOnlySet, ExpiringFWWRegister, ExpiringSet, SizedFWWExpiringSet, CrdtMap, Fww, Lww, Crdt},
     framework::{ExplorableMap, Map}
};

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
        Some(PyList::new(m.py(), path.iter().map(|l| (l.x, l.y))))
    } else {
        None
    };
    Ok(path)
}

#[pyfunction]
pub fn wander<'py>(py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(r) = ::client_utils::behaviors::wander() {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
}

#[pyfunction]
pub fn attack_nearest<'py>(py: Python<'py>, exclude_factions: Vec<i64>) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(r) = ::client_utils::behaviors::attack_nearest(&exclude_factions) {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
}

#[pyfunction]
pub fn convert<'py>(py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(r) = ::client_utils::behaviors::convert() {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
}

fn python_slot_to_rust<'py>(py: Python<'py>, slot: Bound<'py, PyAny>) -> PyResult<EquipmentSlot> {
    let module = py.import("client_utils")?;
    if slot.is_instance(&module.dict().get_item("EquipmentSlot_RightHand")?.unwrap().call0()?)? {
        Ok(EquipmentSlot::RightHand)
    } else if slot.is_instance(&module.dict().get_item("EquipmentSlot_LeftHand")?.unwrap().call0()?)? {
        Ok(EquipmentSlot::LeftHand)
    } else {
        Err(PyTypeError::new_err("Not an EquipmentSlot"))
    }
}

fn rust_slot_to_python<'py>(py: Python<'py>, slot: EquipmentSlot) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import("client_utils")?;
    match slot {
        EquipmentSlot::RightHand => module.dict().get_item("EquipmentSlot_RightHand")?.unwrap().call0(),
        EquipmentSlot::LeftHand => module.dict().get_item("EquipmentSlot_LeftHand")?.unwrap().call0(),
    }
}

#[pyfunction]
pub fn equip<'py>(py: Python<'py>, item: i64, slot: Bound<'py, PyAny>) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(r) = ::client_utils::behaviors::equip(item, python_slot_to_rust(py, slot)?) {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
}

#[pyfunction]
pub fn attack_target<'py>(py: Python<'py>, loc: Bound<'py, PyAny>) -> PyResult<Option<Bound<'py, PyAny>>> {
    let x:i32 = loc.get_item("x")?.extract()?;
    let y:i32 = loc.get_item("y")?.extract()?;
    if let Some(r) = ::client_utils::behaviors::attack_target(Loc { x,y }) {
        Ok(Some(rust_command_to_py_command(r, py)?))
    } else {
        Ok(None)
    }
}

fn rust_direction_to_python<'py>(py: Python<'py>, dir: Direction) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import("client_utils")?;
    match dir {
        Direction::North => module.dict().get_item("Direction_North")?.unwrap().call0(),
        Direction::NorthEast => module.dict().get_item("Direction_NorthEast")?.unwrap().call0(),
        Direction::East => module.dict().get_item("Direction_East")?.unwrap().call0(),
        Direction::SouthEast => module.dict().get_item("Direction_SouthEast")?.unwrap().call0(),
        Direction::South => module.dict().get_item("Direction_South")?.unwrap().call0(),
        Direction::SouthWest => module.dict().get_item("Direction_SouthWest")?.unwrap().call0(),
        Direction::West => module.dict().get_item("Direction_West")?.unwrap().call0(),
        Direction::NorthWest => module.dict().get_item("Direction_NorthWest")?.unwrap().call0(),
    }
}

fn rust_action_target_to_py_action_target<'py>(target: ActionTarget, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import("client_utils")?;
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
        ActionTarget::EquipmentSlotAndItem((slot, item)) => {
            let slot = rust_slot_to_python(py, slot)?;
            let target_class = module.dict().get_item("ActionTarget_EquipmentSlotAndItems")?.unwrap();
            target_class.call1((slot, item,))
        }
        ActionTarget::Creature(id) => {
            module.dict().get_item("ActionTarget_Creature")?.unwrap().call1((id,))
        }
        ActionTarget::Actor => {
            module.dict().get_item("ActionTarget_Actor")?.unwrap().call0()
        }
        ActionTarget::EquipmentSlot(slot) => {
            let slot = rust_slot_to_python(py, slot)?;
            module.dict().get_item("ActionTarget_EquipmentSlot")?.unwrap().call1((slot,))
        }
        ActionTarget::Direction(dir)=> {
            let dir = rust_direction_to_python(py, dir)?;
            module.dict().get_item("ActionTarget_Direction")?.unwrap().call1((dir,))
        }
    }
}

fn rust_command_to_py_command<'py>(command: Command, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    let module = py.import("client_utils")?;
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

#[pyclass(module="client_utils", name="ExpiringFWWRegister")]
#[derive(Default)]
struct PyExpiringFWWRegister(ExpiringFWWRegister<PyAnyCmp>);

#[pymethods]
impl PyExpiringFWWRegister {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get<'py>(&self, py: Python<'py>) -> Option<&Bound<'py, PyAny>> {
        self.0.get().map(|v| v.0.bind(py))
    }

    pub fn set<'py>(&mut self, value: Bound<'py, PyAny>, now: i64, expires: i64) {
        self.0.set(PyAnyCmp(value.unbind()), now, expires);
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let d = PyDict::new(py);
        d.set_item("value", self.0.value.clone().map(|v| v.0))?;
        d.set_item("written", self.0.written)?;
        d.set_item("expires", self.0.expires)?;
        Ok(d.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyDict>) -> PyResult<()> {
        let d = state.get_item("value")?.unwrap();
        if d.is_none() {
            self.0.value = None;
        } else {
            self.0.value = Some(PyAnyCmp(d.extract()?));
        }
        self.0.written = state.get_item("written")?.unwrap().extract()?;
        self.0.expires = state.get_item("expires")?.unwrap().extract()?;
        Ok(())
    }
}


#[pyclass(module="client_utils", name="GrowOnlySet")]
#[derive(Default)]
struct PyGrowOnlySet(GrowOnlySet<PyAnyCmp>);

#[pymethods]
impl PyGrowOnlySet {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<'py>(&mut self, value: Bound<'py, PyAny>) {
        self.0.insert(PyAnyCmp(value.unbind()));
    }

    pub fn contains<'py>(&mut self, value: Bound<'py, PyAny>) -> bool {
        self.0.contains(&PyAnyCmp(value.unbind()))
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let d = PySet::empty(py)?;
        for v in &(self.0).0 {
            d.add(v.0.clone_ref(py))?;
        }
        Ok(d.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PySet>) -> PyResult<()> {
        for v in state.iter() {
            (self.0).0.insert(PyAnyCmp(v.unbind()));
        }
        Ok(())
    }
}

#[pyclass(module="client_utils", name="ExpiringSet")]
#[derive(Default)]
struct PyExpiringSet(ExpiringSet<PyAnyCmp>);

#[pymethods]
impl PyExpiringSet {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<'py>(&mut self, value: Bound<'py, PyAny>, expires: i64) {
        self.0.insert(PyAnyCmp(value.unbind()), expires);
    }

    pub fn contains<'py>(&mut self, value: Bound<'py, PyAny>) -> bool {
        self.0.contains(&PyAnyCmp(value.unbind()))
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let d = PyDict::new(py);
        for (k,v) in &(self.0).0 {
            d.set_item(k.0.clone_ref(py), v)?;
        }
        Ok(d.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyDict>) -> PyResult<()> {
        for (k,v) in state.iter() {
            (self.0).0.insert(PyAnyCmp(k.unbind()),v.extract()?);
        }
        Ok(())
    }
}


#[pyclass(module="client_utils", name="SizedFWWExpiringSet")]
struct PySizedFWWExpiringSet(SizedFWWExpiringSet<PyAnyCmp>);

#[pymethods]
impl PySizedFWWExpiringSet {
    #[new]
    pub fn new() -> Self {
        Self(SizedFWWExpiringSet::new(10))
    }

    #[staticmethod]
    pub fn with_size(size: usize) -> Self {
        Self(SizedFWWExpiringSet::new(size))
    }

    pub fn insert<'py>(&mut self, value: Bound<'py, PyAny>, now: i64, expires: i64) {
        self.0.insert(PyAnyCmp(value.unbind()), now, expires);
    }

    pub fn contains<'py>(&mut self, value: Bound<'py, PyAny>) -> bool {
        self.0.contains(&PyAnyCmp(value.unbind()))
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let inner = PyDict::new(py);
        for (k,v) in &(self.0).0 {
            inner.set_item(k.0.clone_ref(py), v)?;
        }
        let outer = PyDict::new(py);
        outer.set_item("size", self.0.1)?;
        outer.set_item("data", inner)?;
        Ok(outer.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyDict>) -> PyResult<()> {
        let data: Bound<PyDict> = state.get_item("data")?.unwrap().extract()?;
        for (k,v) in data.iter() {
            (self.0).0.insert(PyAnyCmp(k.unbind()),v.extract()?);
        }
        (self.0).1 = state.get_item("size")?.unwrap().extract()?;
        Ok(())
    }
}

#[pyclass(module="client_utils", name="FwwCrdtMap")]
#[derive(Default)]
struct PyFwwCrdtMap(CrdtMap<PyAnyCmp, PyAnyCmp, Fww>);

#[pymethods]
impl PyFwwCrdtMap {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<'py>(&mut self, key: Bound<'py, PyAny>, value: Bound<'py, PyAny>, now: i64) {
        self.0.insert(PyAnyCmp(key.unbind()), PyAnyCmp(value.unbind()), now);
    }

    pub fn contains_key<'py>(&mut self, key: Bound<'py, PyAny>) -> bool {
        self.0.contains_key(&PyAnyCmp(key.unbind()))
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let d = PyDict::new(py);
        for (k,v) in &(self.0).0 {
            d.set_item(k.0.clone_ref(py), ((v.0).0.clone_ref(py), v.1))?;
        }
        Ok(d.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyDict>) -> PyResult<()> {
        for (k,v) in state.iter() {
            (self.0).0.insert(PyAnyCmp(k.unbind()),(PyAnyCmp(v.get_item(0)?.unbind()), v.get_item(1)?.extract()?));
        }
        Ok(())
    }
}

#[pyclass(module="client_utils", name="LwwCrdtMap")]
#[derive(Default)]
struct PyLwwCrdtMap(CrdtMap<PyAnyCmp, PyAnyCmp, Lww>);

#[pymethods]
impl PyLwwCrdtMap {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<'py>(&mut self, key: Bound<'py, PyAny>, value: Bound<'py, PyAny>, now: i64) {
        self.0.insert(PyAnyCmp(key.unbind()), PyAnyCmp(value.unbind()), now);
    }

    pub fn contains_key<'py>(&mut self, key: Bound<'py, PyAny>) -> bool {
        self.0.contains_key(&PyAnyCmp(key.unbind()))
    }

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }

    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let d = PyDict::new(py);
        for (k,v) in &(self.0).0 {
            d.set_item(k.0.clone_ref(py), ((v.0).0.clone_ref(py), v.1))?;
        }
        Ok(d.to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyDict>) -> PyResult<()> {
        for (k,v) in state.iter() {
            (self.0).0.insert(PyAnyCmp(k.unbind()),(PyAnyCmp(v.get_item(0)?.unbind()), v.get_item(1)?.extract()?));
        }
        Ok(())
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
            let module = py.import("client_utils")?;
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

    pub fn merge<'py>(&mut self, other: &Self) -> PyResult<()> {
        self.0.merge(&other.0).unwrap();
        Ok(())
    }

    pub fn cleanup(&mut self, now: i64) {
        self.0.cleanup(now);
    }


    pub fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyBytes::new(py, &bincode::serialize(&self.0).unwrap()).to_object(py))
    }

    pub fn __setstate__(&mut self, state: Bound<PyBytes>) -> PyResult<()> {
        self.0 = bincode::deserialize(state.as_bytes()).unwrap();
        Ok(())
    }
}

#[pymodule(name = "client_utils")]
fn client_utils(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(astar, module)?)?;
    module.add_function(wrap_pyfunction!(convert, module)?)?;
    module.add_function(wrap_pyfunction!(equip, module)?)?;
    module.add_function(wrap_pyfunction!(attack_nearest, module)?)?;
    module.add_function(wrap_pyfunction!(attack_target, module)?)?;
    module.add_function(wrap_pyfunction!(wander, module)?)?;

    module.add_class::<PyExpiringFWWRegister>()?;
    module.add_class::<PyExpiringSet>()?;
    module.add_class::<PyGrowOnlySet>()?;
    module.add_class::<PySizedFWWExpiringSet>()?;
    module.add_class::<PyFwwCrdtMap>()?;
    module.add_class::<PyLwwCrdtMap>()?;
    module.add_class::<PyExplorableMap>()?;

    let code = include_str!("defs.py");
    let defs = PyModule::from_code(module.py(), CString::new(code)?.as_c_str(), c"defs.py", c"defs")?;
    for (k,v) in defs.dict() {
        if let Ok(k) = k.extract::<&str>() {
            if !k.starts_with("__") {
                module.add(k, v)?;
            }
        }
    }

    Ok(())
}
