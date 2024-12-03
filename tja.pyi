from enum import Enum
from typing import Dict, List, Optional

class ParsingMode(Enum):
    MetadataOnly = "MetadataOnly"
    MetadataAndHeader = "MetadataAndHeader"
    Full = "Full"

class PyNote:
    note_type: str
    timestamp: float
    scroll: float
    delay: float
    bpm: float
    gogo: bool

class PySegment:
    measure_num: int
    measure_den: int
    barline: bool
    branch: Optional[str]
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

def parse_tja(content: str, mode: ParsingMode = ParsingMode.Full) -> PyParsedTJA: ... 
