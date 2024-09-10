from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from ..types import Result, Ok, Err, Some


@dataclass
class Loc:
    x: int
    y: int

@dataclass
class Tile:
    passable: bool
    opaque: bool
    name: str

@dataclass
class Creature:
    name: str
    id: int
    faction: int
    broadcast: Optional[bytes]

@dataclass
class Item:
    id: int
    name: str
    is_furniture: bool
    is_passable: bool
    metadata: Optional[str]


@dataclass
class EquipmentSlot_LeftHand:
    pass


@dataclass
class EquipmentSlot_RightHand:
    pass


EquipmentSlot = Union[EquipmentSlot_LeftHand, EquipmentSlot_RightHand]



@dataclass
class BuffDurability_Transient:
    pass


@dataclass
class BuffDurability_DecreasePerTurn:
    value: int


@dataclass
class BuffDurability_Permanent:
    pass


BuffDurability = Union[BuffDurability_Transient, BuffDurability_DecreasePerTurn, BuffDurability_Permanent]



@dataclass
class Direction_North:
    pass


@dataclass
class Direction_NorthWest:
    pass


@dataclass
class Direction_NorthEast:
    pass


@dataclass
class Direction_South:
    pass


@dataclass
class Direction_SouthWest:
    pass


@dataclass
class Direction_SouthEast:
    pass


@dataclass
class Direction_East:
    pass


@dataclass
class Direction_West:
    pass


Direction = Union[Direction_North, Direction_NorthWest, Direction_NorthEast, Direction_South, Direction_SouthWest, Direction_SouthEast, Direction_East, Direction_West]


@dataclass
class AttackParams:
    amount: int
    range: int

@dataclass
class ApplyBuffParams:
    range: int
    name: str
    amount: int
    durability: BuffDurability

@dataclass
class HaulParams:
    strength: int


@dataclass
class ActionTarget_Creature:
    value: int


@dataclass
class ActionTarget_Actor:
    pass


@dataclass
class ActionTarget_EquipmentSlot:
    value: EquipmentSlot


@dataclass
class ActionTarget_EquipmentSlotAndItem:
    value: Tuple[EquipmentSlot, int]


@dataclass
class ActionTarget_Direction:
    value: Direction


@dataclass
class ActionTarget_Items:
    value: List[int]


@dataclass
class ActionTarget_Location:
    value: Loc


ActionTarget = Union[ActionTarget_Creature, ActionTarget_Actor, ActionTarget_EquipmentSlot, ActionTarget_EquipmentSlotAndItem, ActionTarget_Direction, ActionTarget_Items, ActionTarget_Location]



@dataclass
class Command_UseAction:
    value: Tuple[int, Optional[ActionTarget]]


@dataclass
class Command_Nothing:
    pass


Command = Union[Command_UseAction, Command_Nothing]


@dataclass
class EquipmentState:
    left_hand: Optional[int]
    right_hand: Optional[int]

@dataclass
class Buff:
    name: str
    amount: int
    durability: BuffDurability

@dataclass
class GameState:
    turn: int
    level_id: int
    level_is_stable: bool


@dataclass
class Key_Up:
    pass


@dataclass
class Key_Down:
    pass


@dataclass
class Key_Left:
    pass


@dataclass
class Key_Right:
    pass


@dataclass
class Key_Space:
    pass


Key = Union[Key_Up, Key_Down, Key_Left, Key_Right, Key_Space]


@dataclass
class AttackDescription:
    initiator: Creature
    initiator_location: Loc
    amount: int

@dataclass
class ConvertParams:
    input: List[Tuple[str, int]]
    output_items: List[str]
    output_resources: List[Tuple[str, int]]


@dataclass
class MicroAction_Walk:
    pass


@dataclass
class MicroAction_Haul:
    value: HaulParams


@dataclass
class MicroAction_Attack:
    value: AttackParams


@dataclass
class MicroAction_ApplyBuff:
    value: ApplyBuffParams


@dataclass
class MicroAction_Convert:
    value: ConvertParams


@dataclass
class MicroAction_Equip:
    pass


@dataclass
class MicroAction_Unequip:
    pass


@dataclass
class MicroAction_PickUp:
    pass


@dataclass
class MicroAction_AbandonLevel:
    pass


MicroAction = Union[MicroAction_Walk, MicroAction_Haul, MicroAction_Attack, MicroAction_ApplyBuff, MicroAction_Convert, MicroAction_Equip, MicroAction_Unequip, MicroAction_PickUp, MicroAction_AbandonLevel]


@dataclass
class Action:
    name: str
    micro_actions: List[MicroAction]

@dataclass
class InventoryItem:
    name: str
    id: int
    level: int
    actions: List[Action]
    resources: Optional[List[Tuple[str, int]]]

@dataclass
class Stats:
    strength: int
    hp: int
    speed: int
    inventory_size: int

@dataclass
class CharacterStats:
    max: Stats
    current: Stats


@dataclass
class Event_Moved:
    value: Tuple[int, int]


@dataclass
class Event_Hauled:
    value: Tuple[int, int]


@dataclass
class Event_Attacked:
    value: AttackDescription


@dataclass
class Event_AddInventoryItem:
    value: int


@dataclass
class Event_RemoveInventoryItem:
    value: int


@dataclass
class Event_EquipItem:
    value: Tuple[EquipmentSlot, int]


@dataclass
class Event_UnequipItem:
    value: EquipmentSlot


@dataclass
class Event_GainResource:
    value: List[Tuple[str, int]]


@dataclass
class Event_ChangedLevel:
    pass


Event = Union[Event_Moved, Event_Hauled, Event_Attacked, Event_AddInventoryItem, Event_RemoveInventoryItem, Event_EquipItem, Event_UnequipItem, Event_GainResource, Event_ChangedLevel]



