from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from .types import Result, Ok, Err, Some
from .imports import types

def tile_at(l: types.Loc) -> Optional[types.Tile]:
    raise NotImplementedError

def visible_tiles() -> List[Tuple[types.Loc, types.Tile]]:
    raise NotImplementedError

def creature_at(l: types.Loc) -> Optional[types.Creature]:
    raise NotImplementedError

def actor() -> Tuple[types.Loc, types.Creature]:
    raise NotImplementedError

def visible_creatures() -> List[Tuple[types.Loc, types.Creature]]:
    raise NotImplementedError

def item_at(l: types.Loc) -> Optional[types.Item]:
    raise NotImplementedError

def visible_items() -> List[Tuple[types.Loc, types.Item]]:
    raise NotImplementedError

def inventory() -> List[types.InventoryItem]:
    raise NotImplementedError

def get_equipment_state() -> types.EquipmentState:
    raise NotImplementedError

def get_character_stats() -> types.CharacterStats:
    raise NotImplementedError

def character_buffs() -> List[types.Buff]:
    raise NotImplementedError

def get_game_state() -> types.GameState:
    raise NotImplementedError

def actions() -> List[types.Action]:
    raise NotImplementedError

def load_store() -> bytes:
    raise NotImplementedError

def save_store(store: bytes) -> None:
    raise NotImplementedError

def broadcast(data: Optional[bytes]) -> None:
    raise NotImplementedError

def highlight_tiles(tiles: List[types.Loc]) -> None:
    raise NotImplementedError

def highlight_actor(color: Optional[Tuple[float, float, float]]) -> None:
    raise NotImplementedError

def events() -> List[types.Event]:
    raise NotImplementedError

def config_data() -> Optional[bytes]:
    raise NotImplementedError

def editor_debug(data: bytes) -> None:
    raise NotImplementedError


class AutoRogueAi(Protocol):

    @abstractmethod
    def editor_config(self) -> Optional[bytes]:
        raise NotImplementedError

    @abstractmethod
    def step(self) -> types.Command:
        raise NotImplementedError

