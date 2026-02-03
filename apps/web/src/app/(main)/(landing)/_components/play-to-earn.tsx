import CreateRoom from "./create-room";
import Gameplay from "./gameplay";
import WinStx from "./win-stx";

export default function PlayToEarn() {
	return (
		<div className="grid grid-cols-1 md:grid-cols-2 md:grid-cols-[1fr_1.5fr] gap-4 mt-16 mx-auto p-4 w-full">
			<CreateRoom />
			<Gameplay />
			<WinStx />
		</div>
	);
}
