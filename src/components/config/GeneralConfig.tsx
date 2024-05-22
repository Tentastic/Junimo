import {useEffect, useState} from "react";
import { Input } from "@components/ui/input"
import { Label } from "@components/ui/label"
import { Folder } from 'lucide-react';
import { Config as ConfigModel } from "@models/config";
import {invoke} from "@tauri-apps/api/core";
import {User} from "@models/user";
import NexusMods from "@assets/NexusMods.png";
import {Switch} from "@components/ui/switch.tsx";
import { open } from '@tauri-apps/plugin-dialog';

export default function GeneralConfig() {
    const [savedConfig, setSavedConfig] = useState<ConfigModel | undefined>();
    const [gamePath, setGamePath] = useState("");
    const [user, setUser] = useState<User>();
    const [nmxSwitch, setNmxSwitch] = useState(false);

    async function loadConfig() {
        try {
            const configPath = await invoke<string>('config_path');
            const config = await invoke<ConfigModel>('get_config', {path: configPath});
            setGamePath(config.game_path);
            setNmxSwitch(config.handle_nxm);
            setSavedConfig(config);
        } catch (error) {
            console.error('Failed to fetch configuration path:', error);
        }
    }

    async function loadUser() {
        const user = await invoke<User | null>('load_user');
        if (user !== null)
            setUser(user);
    }

    async function fetchConfigPath() {
        const path = await open({
            multiple: false,
            directory: true,
        });
        if (path !== null) {
            setGamePath(path);
        }
    }

    async function save() {
        const configPath = await invoke<string>('config_path');
        if (savedConfig !== undefined) {
            const config : ConfigModel = {
                init_app: true,
                game_path: gamePath,
                handle_nxm: nmxSwitch,
                activate_requirements: savedConfig.activate_requirements,
                block_on_missing_requirements: savedConfig.block_on_missing_requirements,
                activate_broken: savedConfig.activate_broken,
                block_on_broken: savedConfig.block_on_broken
            }

            const path = await invoke<ConfigModel>('save_config_button', {config: config, path: configPath});
            setGamePath(path.game_path);
        }
    }

    async function loginUser() {
        const user = await invoke<User | null>('connect_user');
        if (user !== null)
            setUser(user);
    }

    async function logoutUser() {
        await invoke('disconnect_user');
        setUser(undefined);
    }

    async function registerNxm() {
        await invoke('register_nxm');
    }

    useEffect(() => {
        if (nmxSwitch) {
            registerNxm();
        }
    }, [nmxSwitch]);

    useEffect(() => {
        loadConfig();
        loadUser();
    }, []);

    return (
        <>
            <div className="absolute right-5">
                <button onClick={save}
                        className="p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white">Save
                </button>
            </div>
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">General Settings</h1>
            <div className="w-full flex flex-col gap-4">
                <div className="flex flex-col">
                    <Label htmlFor="path" className="ml-1 mb-2 text-xl">Game Path</Label>
                    <p className="text-sm text-muted-foreground"></p>
                    <div className="flex gap-2">
                        <Input id="path" placeholder="Please enter your game path..."
                               value={gamePath} onChange={x => setGamePath(x.target.value)}/>
                        <button onClick={fetchConfigPath} className="w-10 h-10 flex items-center justify-center transition duration-150 border rounded-lg
                        bg-muted hover:bg-muted-dark">
                            <Folder size={20}/>
                        </button>
                    </div>
                </div>
                <div className="w-full h-[2px] border-lg bg-muted"/>
                <div>
                    {user === undefined || user === null ? (
                            <div className="flex gap-1 items-center">
                                Login via
                                <button onClick={loginUser}
                                        className="p-1 px-2 ml-2 flex gap-2 transition duration-150 bg-amber-600 hover:bg-amber-700 rounded font-bold">
                                    <img src={NexusMods} alt="NexusMods Logo" className="h-6 w-6"/>
                                    Nexus Mods
                                </button>
                            </div>
                        ) :
                        (
                            <div className="flex flex-col gap-2">
                                <h2 className="text-xl">NexusMods Settings</h2>
                                <div className="flex gap-2 items-center justify-between rounded-lg p-2 bg-muted">
                                    <div className="flex items-center gap-2 text-lg">
                                        <img src={NexusMods} alt="NexusMods Logo" className="h-8 w-8"/>
                                        <p>Logged in as: <span className="font-bold">{user.name}</span></p>
                                    </div>
                                    <button onClick={logoutUser}
                                            className="p-1 px-5 flex gap-2 transition duration-150 bg-destructive hover:brightness-75 rounded">
                                        Logout
                                    </button>
                                </div>
                                <div className="flex gap-2 items-center mt-4">
                                    <Switch checked={nmxSwitch} onCheckedChange={x => setNmxSwitch(!nmxSwitch)}/>
                                    Let Junimo handle the
                                    <div className="bg-[#d98f40] p-1 px-2 w-fit flex gap-1">
                                        <img src={NexusMods} alt="Nexus Mods Mod Manager Icon" className="h-full w-6"/>
                                        MOD MANAGER DOWNLOAD
                                    </div>
                                    Button
                                </div>
                            </div>
                        )}
                </div>
            </div>
        </>
    )
}