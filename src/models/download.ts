export interface Download {
    name: string;
    size: number;
    current: number;
    aborted: boolean;
    finished: boolean;
}