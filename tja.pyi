from enum import Enum
from typing import Any, Dict, List, Optional, Literal

PyChartCourse = Literal["Easy", "Normal", "Hard", "Oni", "Ura"]
PyNoteType = Literal[
    "Don", "Ka", "DonBig", "KaBig", "Roll", "RollBig", "Balloon", "EndOf", "BalloonAlt"
]

class PyParsingMode(Enum):
    MetadataOnly = "MetadataOnly"
    MetadataAndHeader = "MetadataAndHeader"
    Full = "Full"
    FullWithBlanks = "FullWithBlanks"
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class PyCourse(Enum):
    Easy = "Easy"
    Normal = "Normal"
    Hard = "Hard"
    Oni = "Oni"
    Ura = "Ura"
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class PyNote:
    note_type: PyNoteType
    timestamp: float
    scroll: float
    delay: float
    bpm: float
    gogo: bool
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...

class PySegment:
    timestamp: float
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

class PyAudioData:
    samples: List[float]
    sample_rate: int
    def __init__(self, samples: List[float], sample_rate: int) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...
    def get_samples(self) -> List[float]: ...
    def get_sample_rate(self) -> int: ...

class PyParsedTJA:
    metadata: Dict[str, str]
    charts: List[PyChart]
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def export(self) -> Dict[str, Any]: ...
    def synthesize_audio(
        self,
        music_data: PyAudioData,
        don_data: PyAudioData,
        ka_data: PyAudioData,
        course: PyCourse,
        branch: Optional[str] = None,
    ) -> PyAudioData: ...

def parse_tja(
    content: str, mode: PyParsingMode = PyParsingMode.Full
) -> PyParsedTJA: ...
def synthesize_tja_audio_py(
    tja: PyParsedTJA,
    music_data: PyAudioData,
    don_data: PyAudioData,
    ka_data: PyAudioData,
    course: PyCourse,
    branch: Optional[str] = None,
) -> PyAudioData: ...
