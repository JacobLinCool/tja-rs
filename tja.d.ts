export function parse_tja(content: string, mode?: WasmParsingMode): Promise<ParsedTJA>;

export enum WasmParsingMode {
    MetadataOnly = 0,
    MetadataAndHeader = 1,
    Full = 2,
}

export interface ParsedTJA {
    metadata: Record<string, string>;
    charts: Chart[];
}

export interface Chart {
    player: number;
    course?: "Easy" | "Normal" | "Hard" | "Oni" | "Ura";
    level?: number;
    balloons: number[];
    headers: Record<string, string>;
    segments: Segment[];
}

export interface Segment {
    timestamp: number;
    measure_num: number;
    measure_den: number;
    barline: boolean;
    branch?: string;
    branch_condition?: string;
    notes: Note[];
}

export interface Note {
    note_type: "Empty" | "Don" | "Ka" | "DonBig" | "KaBig" | "Roll" | "RollBig" | "Balloon" | "EndOf" | "BalloonAlt";
    timestamp: number;
    scroll: number;
    delay: number;
    bpm: number;
    gogo: boolean;
}
