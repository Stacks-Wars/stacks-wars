"use client";

import { Button } from "@/components/ui/button";
import useCopyToClipboard from "@/lib/hooks/useCopy";
import { useLobby } from "@/lib/stores/room";
import { Share } from "lucide-react";
import { toast } from "sonner";

export default function ShareButton() {
	const [copiedtext, copy] = useCopyToClipboard();
	const lobby = useLobby();
	return (
		<Button
			size="sm"
			className="shrink-0 gap-2 rounded-full px-5 py-2.5 has-[>svg]:px-5"
			onClick={() => {
				copy(`https://www.stackswars.com/room/${lobby?.path}`);
				toast.info(`Room link copied to clipboard!`);
			}}
		>
			<Share className="size-4" />
			<span>Share</span>
		</Button>
	);
}
