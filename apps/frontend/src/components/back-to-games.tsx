import { ArrowLeft } from "lucide-react";
import Link from "next/link";

export default function BackToGames() {
	return (
		<Link
			href="/games"
			className="text-muted-foreground hover:text-foreground mb-4 inline-flex items-center gap-1 text-sm font-medium transition-colors sm:mb-6"
		>
			<ArrowLeft className="h-4 w-4" />
			<span>Back to Games</span>
		</Link>
	);
}
