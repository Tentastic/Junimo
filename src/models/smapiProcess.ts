export default interface SmapiProcess {
    started: boolean,
    download_progress: number,
    size: number,
    download_finished: boolean,
    installation_started: boolean,
    installation_finished: boolean,
}