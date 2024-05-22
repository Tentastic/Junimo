import React, {useEffect, useState} from "react";
import { Input } from "@components/ui/input"
import { Label } from "@components/ui/label"
import { Folder } from 'lucide-react';
import { Config as ConfigModel } from "@models/config";
import {invoke} from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import {Select, SelectContent, SelectItem, SelectTrigger, SelectValue} from "@components/ui/select.tsx";
import {Checkbox} from "@components/ui/checkbox.tsx";
import Save from "@components/ui/save.tsx";
import {useTranslation} from "react-i18next";

export default function GeneralConfig() {
    const [savedConfig, setSavedConfig] = useState<ConfigModel | undefined>();
    const [gamePath, setGamePath] = useState("");
    const [lang, setLang] = useState("en");
    const [keepOpen, setKeepOpen] = useState(true);

    const { t, i18n } = useTranslation("config");

    async function loadConfig() {
        try {
            const configPath = await invoke<string>('config_path');
            const config = await invoke<ConfigModel>('get_config', {path: configPath});
            setGamePath(config.game_path);
            if (config.lang !== null)
                setLang(config.lang);
            if (config.keep_open !== null)
                setKeepOpen(config.keep_open);
            setSavedConfig(config);
        } catch (error) {
            console.error('Failed to fetch configuration path:', error);
        }
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
                handle_nxm: savedConfig.handle_nxm,
                activate_requirements: savedConfig.activate_requirements,
                block_on_missing_requirements: savedConfig.block_on_missing_requirements,
                activate_broken: savedConfig.activate_broken,
                block_on_broken: savedConfig.block_on_broken,
                lang: lang,
                keep_open: keepOpen
            }

            const path = await invoke<ConfigModel>('save_config_button', {config: config, path: configPath});
            setGamePath(path.game_path);
            i18n.changeLanguage(lang).then(r => console.log(r)).catch(e => console.error(e));
        }
    }

    useEffect(() => {
        loadConfig();
    }, []);

    return (
        <>
            <div className="absolute right-5">
                <div onClick={save}>
                    <Save />
                </div>
            </div>
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">{t("generalTitle")}</h1>
            <div className="w-full flex flex-col gap-4">
                <div className="flex flex-col">
                    <Label htmlFor="path" className="ml-1 mb-2 text-xl">{t("gamePath")}</Label>
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

                <div className="flex flex-col">
                    <Label htmlFor="path" className="mb-1 text-xl">{t("language")}</Label>
                    <div className="flex gap-2">
                        <Select value={lang} onValueChange={e => setLang(e)}>
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="English"/>
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="en">English</SelectItem>
                                <SelectItem value="de">German</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
                <div className="w-full h-[2px] border-lg bg-muted"/>

                <div className="flex flex-col">
                    <Label htmlFor="path" className="mb-1 text-xl">{t("keepOpen")}</Label>
                    <p className="text-sm text-muted-foreground mb-2">{t("keepOpenDesc")}</p>
                    <div className="flex gap-2">
                        <Checkbox checked={keepOpen} onCheckedChange={e => setKeepOpen(!keepOpen)}/>
                    </div>
                </div>
            </div>
        </>
    )
}