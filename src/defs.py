from auto_rogue_ai import *
from auto_rogue_ai.imports.types import *
class Bot:
    def __init__(self):
        self.map = None

    def __step__(self) -> Command:
        if self.map is not None:
            self.map.update()
        return self.step()

    def step(self) -> Command:
        Command_Nothing()

def find_action(ty, pool: list[Action] = None) -> Optional[int]:
    if pool is None:
        pool = actions()
    for i, action in enumerate(pool):
        for m in action.micro_actions:
            if isinstance(m, ty):
                return (i, action, m)
