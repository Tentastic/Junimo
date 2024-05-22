import React, { createContext, useState, useContext } from 'react';
import {Profile} from "@models/profile.ts";
import {ModInfos} from "@models/mods.ts";
import {invoke} from "@tauri-apps/api/core";


// Define the type for the context value
interface ModsContextType {
    appKey: [number, React.Dispatch<React.SetStateAction<number>>];
    reloadKey: [number, React.Dispatch<React.SetStateAction<number>>];
    profile: [Profile | undefined, React.Dispatch<React.SetStateAction<Profile | undefined>>];
    activatedMods: [Map<string, ModInfos[]>, React.Dispatch<React.SetStateAction<Map<string, ModInfos[]>>>];
    installedMods: [ModInfos[], React.Dispatch<React.SetStateAction<ModInfos[]>>];
    selectedAdd: [string[], React.Dispatch<React.SetStateAction<string[]>>];
    selectedRemove: [string[], React.Dispatch<React.SetStateAction<string[]>>];
    addMods: () => void;
    removeMods: () => void;
}

export const ModsContext = createContext<ModsContextType | undefined>(undefined);

export const useModsState = () => {
    const context = useContext(ModsContext);
    if (!context) {
        throw new Error('useModsState must be used within a ModsProvider');
    }
    return context;
};

const ModsProvider = ({ children }: { children: React.ReactNode }) => {
    const [appKey, setAppKey] = useState(0);
    const [key, setKey] = useState(0);
    const [profile, setProfile] = useState<Profile>();
    const [activatedMods, setActivatedMods] = useState<Map<string, ModInfos[]>>(new Map());
    const [modList, setModList] = useState<ModInfos[]>([]);
    const [selectedAdd, setSelectedAdd] = useState<string[]>([]);
    const [selectedRemove, setSelectedRemove] = useState<string[]>([]);

    async function add() {
        console.log("Add");
        if (profile !== undefined) {
            const mods = profile.mods.concat(modList.filter((x, i) => selectedAdd.includes(x.name)));
            const path = await invoke('profile_path');
            await invoke('change_profile_mods', {name: profile.name, mods: mods, path: path});
            setSelectedAdd([]);
            setKey(prevKey => prevKey + 1);
        }
    }

    async function remove() {
        if (profile !== undefined) {
            const mods = profile.mods.filter((x, i) => !selectedRemove.includes(x.name));
            const path = await invoke('profile_path');
            await invoke('change_profile_mods', {name: profile.name, mods: mods, path: path});
            setSelectedRemove([]);
            setKey(prevKey => prevKey + 1);
        }
    }

    return (
        <ModsContext.Provider value={{ appKey: [appKey, setAppKey], reloadKey: [key, setKey], profile: [profile, setProfile], activatedMods: [activatedMods, setActivatedMods],
            installedMods: [modList, setModList], selectedAdd: [selectedAdd, setSelectedAdd], selectedRemove: [selectedRemove, setSelectedRemove],
            addMods: add, removeMods: remove }}>
            {children}
        </ModsContext.Provider>
    );
};

export default ModsProvider;