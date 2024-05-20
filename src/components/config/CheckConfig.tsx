import {Label} from "@components/ui/label.tsx";
import {Input} from "@components/ui/input.tsx";
import {Folder} from "lucide-react";
import NexusMods from "@assets/NexusMods.png";
import {Switch} from "@components/ui/switch.tsx";
import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Config as ConfigModel} from "@models/config.ts";
import {Checkbox} from "@components/ui/checkbox.tsx";
import {useModsState} from "@components/ModsProvider.tsx";
import {clsx} from "clsx";


export default function CheckConfig() {
    const { reloadKey } = useModsState();

    const [savedConfig, setSavedConfig] = useState<ConfigModel | undefined>();
    const [gamePath, setGamePath] = useState("");
    const [activateRequirements, setActivateRequirements] = useState(true);
    const [blockOnMissingRequirements, setBlockOnMissingRequirements] = useState(true);
    const [activateBroken, setActivateBroken] = useState(true);
    const [blockOnBroken, setBlockOnBroken] = useState(true);

    async function loadConfig() {
        try {
            const configPath = await invoke<string>('config_path');
            const config = await invoke<ConfigModel>('get_config', {path: configPath});
            setSavedConfig(config);
            if (config.activate_requirements !== null)
                setActivateRequirements(config.activate_requirements);
            else
                setActivateRequirements(true);

            if (config.block_on_missing_requirements !== null)
                setBlockOnMissingRequirements(config.block_on_missing_requirements);
            else
                setBlockOnMissingRequirements(true);

            if (config.activate_broken !== null)
                setActivateBroken(config.activate_broken);
            else
                setActivateBroken(true);

            if (config.block_on_broken !== null)
                setBlockOnBroken(config.block_on_broken);
            else
                setBlockOnBroken(true);
        } catch (error) {
            console.error('Failed to fetch configuration path:', error);
        }
    }

    async function save() {
        const configPath = await invoke<string>('config_path');
        if (savedConfig !== undefined) {
            const config : ConfigModel = {
                init_app: savedConfig.init_app,
                game_path: savedConfig.game_path,
                handle_nxm: savedConfig.handle_nxm,
                activate_requirements: activateRequirements,
                block_on_missing_requirements: blockOnMissingRequirements,
                activate_broken: activateBroken,
                block_on_broken: blockOnBroken
            }

            const path = await invoke<ConfigModel>('save_config_button', {config: config, path: configPath});
            reloadKey[1](prevKey => prevKey + 1);
        }
    }

    useEffect(() => {
        loadConfig();
    }, []);

    return (
        <>
            <div className="absolute right-5">
                <button onClick={save}
                        className="p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white">Save
                </button>
            </div>
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">Check Settings</h1>
            <div className="w-full flex flex-col gap-4">
                <div className="flex flex-col">
                    <Label htmlFor="path" className="mb-1 text-xl">Activate requirement checks</Label>
                    <p className="text-sm text-muted-foreground mb-2">Junimo will check for missing mod requirements</p>
                    <div className="flex gap-2">
                        <Checkbox checked={activateRequirements}
                                  onCheckedChange={e => setActivateRequirements(!activateRequirements)}/>
                    </div>
                </div>
                <div className={clsx("flex flex-col ml-5", activateRequirements ? "" : "opacity-50")}>
                    <Label htmlFor="path" className="mb-1 text-xl">Block on missing requirements</Label>
                    <p className="text-sm text-muted-foreground mb-2">Block Junimo from starting the game if you miss
                        requirements</p>
                    <div className="flex gap-2">
                        <Checkbox checked={blockOnMissingRequirements}
                                  onCheckedChange={e => setBlockOnMissingRequirements(!blockOnMissingRequirements)}/>
                    </div>
                </div>
                <div className="w-full h-[2px] border-lg bg-muted"/>
                <div className="flex flex-col">
                    <Label htmlFor="path" className="mb-1 text-xl">Activate broken checks</Label>
                    <p className="text-sm text-muted-foreground mb-2">Junimo will check for any mods that are broken in your current version</p>
                    <div className="flex gap-2">
                        <Checkbox checked={activateBroken}
                                  onCheckedChange={e => setActivateBroken(!activateBroken)}/>
                    </div>
                </div>
                <div className={
                    clsx("flex flex-col ml-5", activateBroken ? "" : "opacity-50")
                }>
                    <Label htmlFor="path" className="mb-1 text-xl">Block on broken mods</Label>
                    <p className="text-sm text-muted-foreground mb-2">Block Junimo from starting the game if any mods are broken</p>
                    <div className="flex gap-2">
                        <Checkbox checked={blockOnBroken} onCheckedChange={e => setBlockOnBroken(!blockOnBroken)}/>
                    </div>
                </div>
            </div>
        </>
    )
}