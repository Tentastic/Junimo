import NexusMods from "@assets/NexusMods.png";
import {Switch} from "@components/ui/switch.tsx";
import {invoke} from "@tauri-apps/api/core";
import {Config as ConfigModel} from "@models/config.ts";
import React, {useEffect, useState} from "react";
import {User} from "@models/user.ts";
import Save from "@components/ui/save.tsx";
import {useTranslation} from "react-i18next";
import {Tooltip, TooltipContent, TooltipProvider, TooltipTrigger} from "@components/ui/tooltip.tsx";
import {Label} from "@components/ui/label.tsx";
import {Input} from "@components/ui/input.tsx";
import {Folder} from "lucide-react";
import {clsx} from "clsx";


export default function NexusConfig() {
    const [savedConfig, setSavedConfig] = useState<ConfigModel | undefined>();
    const [user, setUser] = useState<User>();
    const [nmxSwitch, setNmxSwitch] = useState(false);
    const [apiKey, setApiKey] = useState("");

    const { t } = useTranslation("config");

    async function loadConfig() {
        try {
            const configPath = await invoke<string>('config_path');
            const config = await invoke<ConfigModel>('get_config', {path: configPath});
            const foundKey = await invoke<string>('load_api_key');
            setApiKey(foundKey);
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

    async function save() {
        const configPath = await invoke<string>('config_path');
        if (savedConfig !== undefined) {
            const config : ConfigModel = {
                init_app: true,
                game_path: savedConfig.game_path,
                handle_nxm: nmxSwitch,
                activate_requirements: savedConfig.activate_requirements,
                block_on_missing_requirements: savedConfig.block_on_missing_requirements,
                activate_broken: savedConfig.activate_broken,
                block_on_broken: savedConfig.block_on_broken,
                lang: savedConfig.lang,
                keep_open: savedConfig.keep_open
            }

            await invoke('set_api_key', {key: apiKey});
            await invoke<ConfigModel>('save_config_button', {config: config, path: configPath});
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
                <div onClick={save}>
                    <Save/>
                </div>
            </div>
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">{t("nexusModsTitle")}</h1>
            <div className="w-full flex flex-col gap-4">
                <div className="flex flex-col">
                    <Label htmlFor="path" className="ml-1 mb-2 text-xl">{t("apiLabel")}</Label>
                    <p className="text-sm text-muted-foreground"></p>
                    <div className="flex gap-2">
                        <Input id="path" placeholder={t("apiPlaceholder")}
                               value={apiKey} onChange={x => setApiKey(x.target.value)}/>
                    </div>
                </div>
                <div className="w-full h-[2px] border-lg bg-muted"/>
                <div>
                    {user === undefined || user === null ? (
                            <div className="flex gap-1 items-center">
                                Login via
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger>
                                            <button
                                                className="p-1 px-2 ml-2 flex gap-2 transition duration-150 bg-amber-600 hover:bg-amber-700 rounded font-bold brightness-50 cursor-not-allowed">
                                                <img src={NexusMods} alt="NexusMods Logo" className="h-6 w-6"/>
                                                Nexus Mods
                                            </button>
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            <p>{t("nexusmodsNotAllowed")}</p>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            </div>
                        ) :
                        (
                            <div className="flex flex-col gap-2">
                                <div className="flex gap-2 items-center justify-between rounded-lg p-2 bg-muted">
                                    <div className="flex items-center gap-2 text-lg">
                                        <img src={NexusMods} alt="NexusMods Logo" className="h-8 w-8"/>
                                        <p>{t("loggedIn")} <span className="font-bold">{user.name}</span></p>
                                    </div>
                                    <button onClick={logoutUser}
                                            className="p-1 px-5 flex gap-2 transition duration-150 bg-destructive hover:brightness-75 rounded">
                                        Logout
                                    </button>
                                </div>
                            </div>
                        )}
                    <div className={clsx("flex gap-2 items-center mt-4", apiKey === "" && "brightness-50 cursor-not-allowed pointer-events-none")}>
                        <Switch checked={nmxSwitch} onCheckedChange={x => setNmxSwitch(!nmxSwitch)}/>
                        {t("handleMessage")}
                        <div className="bg-[#d98f40] p-1 px-2 w-fit flex gap-1">
                            <img src={NexusMods} alt="Nexus Mods Mod Manager Icon" className="h-full w-6"/>
                            MOD MANAGER DOWNLOAD
                        </div>
                        {t("buttonMessage")}
                    </div>
                </div>
            </div>
        </>
    )
}