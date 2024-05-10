import { ModInfos } from './mods';

export interface Profile {
    name: string,
    mods: ModInfos[],
    currently: boolean
}