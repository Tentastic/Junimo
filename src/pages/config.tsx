import {useEffect, useState} from "react";
import { Input } from "@components/ui/input"
import { Label } from "@components/ui/label"
import { Folder } from 'lucide-react';
import { Config as ConfigModel } from "@models/config";
import {invoke} from "@tauri-apps/api/tauri";
import {User} from "@models/user";
import NexusMods from "@assets/NexusMods.png";
import {Switch} from "@components/ui/switch.tsx";

export default function Config() {
    const [] = useState("");
    const [gamePath, setGamePath] = useState("");
    const [user, setUser] = useState<User>();
    const [nmxSwitch, setNmxSwitch] = useState(false);

    async function loadConfig() {
        try {
            const config = await invoke<ConfigModel>('load_config');
            setGamePath(config.game_path);
            setNmxSwitch(config.handle_nxm);
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
        const path = await invoke<string>('select_game_dir');
        setGamePath(path);
    }

    async function save() {
        const config : ConfigModel = {
            init_app: true,
            game_path: gamePath,
            handle_nxm: nmxSwitch
        }

        const path = await invoke<ConfigModel>('save_config_button', {config: config });
        setGamePath(path.game_path);
    }

    async function loginUser() {
        const user = await invoke<User | null>('connect_user');
        if (user !== null)
            setUser(user);
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
        <div className="w-full h-[100vh] p-6 flex flex-col justify-start items-start">
            <h1 className="text-3xl text-left font-bold mb-4">Setup Configurations</h1>
            <div className="w-full flex flex-col gap-4">
                <div className="flex flex-col">
                    <Label htmlFor="path" className="text-base ml-1">Game Path</Label>
                    <div className="flex gap-2">
                        <Input id="path" placeholder="Please enter your game path..."
                               value={gamePath} onChange={x => setGamePath(x.target.value)} />
                        <button onClick={fetchConfigPath} className="w-10 h-10 flex items-center justify-center transition duration-150 border rounded-lg
                        bg-muted hover:bg-muted-dark">
                            <Folder size={20}/>
                        </button>
                    </div>
                </div>
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
                            <div>
                                <div className="flex gap-2">
                                    <img src={NexusMods} alt="NexusMods Logo" className="h-6 w-6"/>
                                    <p>Logged in as: <span className="font-bold">{user.name}</span></p>
                                </div>
                                <div className="flex gap-2 items-center mt-4">
                                    <Switch checked={nmxSwitch} onCheckedChange={x => setNmxSwitch(!nmxSwitch)} />
                                    Let Junimo handle the
                                    <div className="bg-[#d98f40] p-1 px-2 w-fit flex gap-1">
                                        <img src={NexusMods} alt="Nexus Mods Mod Manager Icon" className="h-full w-6" />
                                        MOD MANAGER DOWNLOAD
                                    </div>
                                    Button
                                </div>
                            </div>
                        )}
                </div>
            </div>
            <div className="h-full w-full flex justify-end items-end">
                <button onClick={save}
                        className="p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white">Save</button>
            </div>
        </div>
    )
}