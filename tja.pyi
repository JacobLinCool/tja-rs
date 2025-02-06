from enum import Enum
from typing import Any, Dict, List, Optional, Literal

PyChartCourse = Literal["Easy", "Normal", "Hard", "Oni", "Ura"]

class PyParsingMode(Enum):
    MetadataOnly = "MetadataOnly"
    MetadataAndHeader = "MetadataAndHeader"
    Full = "Full"
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class PyNote:
    note_type: str
    timestamp: float
    scroll: float
    delay: float
    bpm: float
    gogo: bool
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...

class PySegment:
    measure_num: int
    measure_den: int
    barline: bool
    branch: Optional[str]
    branch_condition: Optional[str]
    notes: List[PyNote]
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...

class PyChart:
    player: int
    course: Optional[PyChartCourse]
    level: Optional[int]
    balloons: List[int]
    headers: Dict[str, str]
    segments: List[PySegment]
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...

class PyParsedTJA:
    metadata: Dict[str, str]
    charts: List[PyChart]
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...

def parse_tja(
    content: str, mode: PyParsingMode = PyParsingMode.Full
) -> PyParsedTJA: ...
