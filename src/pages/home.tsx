import {useContext, useEffect, useRef, useState} from "react";
import DancingJunimo from "../assets/JunimoDance.gif";
import { invoke } from "@tauri-apps/api/core";
import "../App.css";
import {
    Menubar,
} from "@components/ui/menubar"
import Downloader from "@components/Downloader.tsx";
import ModsInstalled from "@components/ModsInstalled.tsx";
import {clsx} from "clsx";
import Mods from "@components/Mods.tsx";
import Console from "@components/Console.tsx";
import {Download} from "@models/download.ts";
import Theme from "@components/menubar/Theme.tsx";
import {listen} from "@tauri-apps/api/event";
import Profiles from "@components/menubar/Profiles.tsx";
import Tools from "@components/menubar/Tools.tsx";
import UtilityBar from "@components/UtilityBar.tsx";
import Help from "@components/menubar/Help.tsx";
import File from "@components/menubar/File.tsx";
import WindowActions from "@components/menubar/WindowActions.tsx";
import {useModsState} from "@components/ModsProvider.tsx";
import MiddleButtons from "@components/MiddleButtons.tsx";
import {useTranslation} from "react-i18next";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

function Home() {
    const [downloadList, setDownloadList] = useState<Download[]>([]);
    const [playing, setPlaying] = useState(false);
    const [submenu, setSubmenu] = useState(false);
    const { t, i18n } = useTranslation('home');

    const { reloadKey } = useModsState();

    window.addEventListener('contextmenu', function (e) {
        e.preventDefault();  // This will prevent the default context menu
    }, false);


  async function initApp() {
      await invoke("init");
      setTimeout(async () => {
          reloadKey[1](prevKey => prevKey + 1);
          await invoke("close_splashscreen");

          const update = await check();
          if (update?.available) {
              await invoke("open_updater");
          }
      }, 400);
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
                    reloadKey[1](prevKey => prevKey + 1);
                    return prevLines.map(x => x.name === data.name ? data : x);
                }

                return prevLines.map(x => x.name === data.name ? data : x);
            });
        };

        const handleReload = (event: any) => {
            reloadKey[1](prevKey => prevKey + 1);
        };

        initApp();

        let unsubscribeEvent = listen('close', handleNewData);
        let unsubscribeDownloadEvent = listen('download', handleDownload);
        let unsubscribeReloadEvent = listen('reload', handleReload);
        let unsubscribeLanguageEvent = listen('language_changed', async (event) => {
            const lang = event.payload as string;
            await i18n.changeLanguage(lang);
        });

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
            unsubscribeDownloadEvent.then((unsub) => unsub());
            unsubscribeReloadEvent.then((unsub) => unsub());
            unsubscribeLanguageEvent.then((unsub) => unsub());
        };
    }, []);

  return (
    <div className="w-full h-[100vh] flex flex-col transition-all duration-300 overflow-y-hidden">
        <div className="pt-0 p-6 flex flex-col h-screen">
            <div>
                <Menubar className="flex justify-between mx-[-18px] mt-[4px]" data-tauri-drag-region>
                    <div className="flex">
                        <File />
                        <Tools />
                        <Profiles setKey={reloadKey[1]} />
                        <Theme />
                        <Help />
                    </div>
                    <div className="w-32 h-full">
                        <WindowActions />
                    </div>
                </Menubar>
                <div className="flex items-center justify-between">
                    <UtilityBar playing={playing} setPlaying={setPlaying} />
                    <div className="w-[30vw] h-10 bg-card rounded-lg border-border border flex justify-around p-1">
                        <button onClick={() => setSubmenu(false)} className={clsx(
                            "flex-grow flex items-center justify-center w-[50%] rounded transition duration-150 cursor-pointer",
                            !submenu ? "bg-background" : "hover:bg-muted"
                        )}>
                            {t("installs")}
                        </button>
                        <button onClick={() => setSubmenu(true)} className={clsx(
                            "flex-grow flex items-center justify-center w-[50%] rounded transition duration-150 cursor-pointer",
                            submenu ? "bg-background" : "hover:bg-muted"
                        )}>
                            {t("downloads")}
                        </button>
                    </div>
                </div>
            </div>

            <div className="flex flex-1 py-4 overflow-hidden" key={reloadKey[0]}>
                <Mods />
                <MiddleButtons playing={playing} />
                {
                    !submenu ?
                        (
                            <ModsInstalled />
                        )
                            :
                        (
                            <Downloader downloadList={downloadList} />
                        )
                }
            </div>

            <Console />
        </div>
    </div>
  );
}

export default Home;
