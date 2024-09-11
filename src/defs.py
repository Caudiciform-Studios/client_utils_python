import pickle

from auto_rogue_ai import *
from auto_rogue_ai.imports.types import *

class Bot:
    def map(self):
        pass

    def broadcast(self):
        pass

    def get_memory(self):
        pass

    def set_memory(self, m):
        pass

    def __step__(self) -> Command:
        try:
            m = pickle.loads(load_store())
            self.set_memory(m)
        except:
            pass

        if (map := self.map()) is not None:
            map.update()

        if (b:= self.broadcast()) is not None:
            (_, a) = actor();
            for _, creature in visible_creatures():
                if a.faction == creature.faction:
                    if creature.broadcast is not None:
                        try:
                            other = pickle.loads(creature.broadcast)
                            b.merge(other)
                        except:
                            pass

        command = self.step()

        if (b:= self.broadcast()) is not None:
            broadcast(pickle.dumps(b))

        if (m := self.get_memory()) is not None:
            save_store(pickle.dumps(m))

        return command

    def step(self) -> Command:
        Command_Nothing()

def find_action(ty, pool: list[Action] = None) -> Optional[int]:
    if pool is None:
        pool = actions()
    for i, action in enumerate(pool):
        for m in action.micro_actions:
            if isinstance(m, ty):
                return (i, action, m)
