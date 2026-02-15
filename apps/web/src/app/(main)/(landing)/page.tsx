import { Suspense } from "react";
import GamesShowcase, {
	GamesShowcaseSkeleton,
} from "./_components/games-showcase";
import Hero from "./_components/hero";
import PlayToEarn from "./_components/play-to-earn";
import TryOut from "./_components/try-out";

export default function HomePage() {
	return (
		<div className="container mx-auto overflow-x-hidden px-4 py-8">
			<Hero />
			<Suspense fallback={<GamesShowcaseSkeleton />}>
				<GamesShowcase />
			</Suspense>
			<PlayToEarn />
			<TryOut />
		</div>
	);
}
