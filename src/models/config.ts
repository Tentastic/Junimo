export interface Config {
    init_app: boolean;
    game_path: string;
    handle_nxm: boolean;
    activate_requirements: boolean | null;
    block_on_missing_requirements: boolean | null;
    activate_broken: boolean | null;
    block_on_broken: boolean | null;
}