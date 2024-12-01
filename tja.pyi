from typing import Dict, List, Optional

class PyNote:
    note_type: str
    scroll: float
    delay: float
    bpm: float
    gogo: bool

class PySegment:
    measure_num: int
    measure_den: int
    barline: bool
    branch_active: bool
    branch_condition: Optional[str]
    notes: List[PyNote]

class PyChart:
    player: int
    course: Optional[str]
    level: Optional[int]
    balloons: List[int]
    headers: Dict[str, str]
    segments: List[PySegment]

class PyParsedTJA:
    metadata: Dict[str, str]
    charts: List[PyChart]

def parse_tja(content: str) -> PyParsedTJA: ... 
