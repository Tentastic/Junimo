import {useEffect, useRef, useState} from "react";
import DancingJunimo from "../assets/JunimoDance.gif";
import { invoke } from "@tauri-apps/api/core";
import "../App.css";
import { ArrowLeft, ArrowRight } from 'lucide-react';
import { Tabs, TabsList, TabsTrigger } from "@components/ui/tabs"
import {
    Menubar,
} from "@components/ui/menubar"
import Downloader from "@components/Downloader.tsx";
import ModsInstalled from "@components/ModsInstalled.tsx";
import {clsx} from "clsx";
import Mods from "@components/Mods.tsx";
import {Profile} from "@models/profile.ts";
import {ModInfos} from "@models/mods.ts";
import Console from "@components/Console.tsx";
import {Download} from "@models/download.ts";
import Theme from "@components/menubar/Theme.tsx";
import {listen} from "@tauri-apps/api/event";
import Profiles from "@components/menubar/Profiles.tsx";
import Tools from "@components/menubar/Tools.tsx";
import UtilityBar from "@components/UtilityBar.tsx";
import Help from "@components/menubar/Help.tsx";
import File from "@components/menubar/File.tsx";

function Home() {
    const [key, setKey] = useState(0);
    const [selectedInstalled, setSelectedInstalled] = useState<number[]>([]);
    const [selectedMods, setSelectedMods] = useState<number[]>([]);
    const [profile, setProfile] = useState<Profile>();
    const [modList, setModList] = useState<ModInfos[]>([]);
    const [downloadList, setDownloadList] = useState<Download[]>([]);
    const [playing, setPlaying] = useState(false);
    const [bigConsole, setBigConsole] = useState(false);

    window.addEventListener('contextmenu', function (e) {
        e.preventDefault();  // This will prevent the default context menu
    }, false);

  async function remove() {
    if (profile !== undefined) {
        const mods = profile.mods.filter((x, i) => !selectedMods.includes(i));
        await invoke('change_profile_mods', {name: profile.name, mods: mods});
        setSelectedMods([]);
        setKey(prevKey => prevKey + 1);
    }
  }

async function add() {
    if (profile !== undefined) {
        const mods = profile.mods.concat(modList.filter((x, i) => selectedInstalled.includes(i)));
        console.log(mods);
        await invoke('change_profile_mods', {name: profile.name, mods: mods});
        setSelectedInstalled([]);
        setKey(prevKey => prevKey + 1);
    }
  }

  function toggleConsole() {
        if (!playing) {
            setBigConsole(!bigConsole);
        }
  }

    useEffect(() => {
        const handleNewData = (event: any) => {
            setPlaying(false);
        };

        const handleDownload = (event: any) => {
            const data = event.payload as Download;
            setDownloadList(prevLines => {
                const newDownloads = [...prevLines];

                if (prevLines.find(x => x.name === data.name) === undefined) {
                    newDownloads.push(data);
                    return newDownloads;
                }

                if (data.aborted) {
                    return prevLines.filter(x => x.name !== data.name);
                }

                if (data.finished) {
                    setKey(prevKey => prevKey + 1);
                    return prevLines.map(x => x.name === data.name ? data : x);
                }

                return prevLines.map(x => x.name === data.name ? data : x);
            });
        };

        const handleReload = (event: any) => {
            setKey(prevKey => prevKey + 1);
        };

        let unsubscribeEvent = listen('close', handleNewData);
        let unsubscribeDownloadEvent = listen('download', handleDownload);
        let unsubscribeReloadEvent = listen('reload', handleReload);

        setTimeout(() => {
            invoke("close_splashscreen");
        }, 1600);

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
            unsubscribeDownloadEvent.then((unsub) => unsub());
            unsubscribeReloadEvent.then((unsub) => unsub());
        };
    }, []);

  return (
    <div className="w-full h-[100vh] flex flex-col transition-all duration-300 overflow-y-hidden">
        <Menubar>
            <File />
            <Tools />
            <Profiles setKey={setKey} />
            <Theme />
            <Help />
        </Menubar>
        <main className="flex-grow flex-1 p-6 pt-2 grid gap-4 grid-rows-[auto_1fr_auto] grid-cols-[1fr_1fr_auto_1fr]">
            <UtilityBar playing={playing} setPlaying={setPlaying} />
            <Tabs defaultValue="mods" className="row-span-2 flex flex-col mt-2">
                <TabsList>
                <TabsTrigger value="mods">Mods</TabsTrigger>
                    <TabsTrigger value="download">Downloads</TabsTrigger>
                </TabsList>
                <ModsInstalled setKey={setKey} modList={modList} setModList={setModList} selected={selectedInstalled} setSelected={setSelectedInstalled}
                               key={key} className={playing || bigConsole ? "max-h-[30vh]" : "max-h-[50vh]"} />
                <Downloader downloadList={downloadList} className={playing || bigConsole ? "max-h-[30vh]" : ""} />
            </Tabs>
            <Mods setKey={setKey} profile={profile} setProfile={setProfile} selected={selectedMods} setSelected={setSelectedMods} key={key}
                  className={playing || bigConsole ? "max-h-[30vh]" : "max-h-[67vh]"} />
            <div className="flex flex-col gap-2 h-full relative">
                <div className="flex flex-col gap-2 h-full items-center justify-center">
                    <button onClick={add} className={clsx(
                        "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                        selectedInstalled.length === 0 && "opacity-50 pointer-events-none"
                    )}>
                        <ArrowLeft/>
                    </button>
                    <button onClick={remove} className={clsx(
                        "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                        selectedMods.length === 0 && "opacity-50 pointer-events-none"
                    )}>
                        <ArrowRight/>
                    </button>
                </div>
                {playing && (
                    <div className="h-12 w-full absolute -bottom-4">
                        <img src={DancingJunimo} alt="Dancing Junimo"/>
                    </div>
                )}
            </div>

            <div onClick={toggleConsole} className={clsx(
                "h-32 border-border transition duration-150 cursor-pointer hover:bg-muted pl-2 col-span-4 border rounded-lg",
                playing || bigConsole ? "h-[50vh]" : "h-32"
            )}>
                <Console playing={playing} bigConsole={bigConsole}/>
            </div>
        </main>
    </div>
  );
}

export default Home;
