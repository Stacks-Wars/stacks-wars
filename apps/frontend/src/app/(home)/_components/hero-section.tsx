import Image from "next/image";
import { Button } from "@/components/ui/button";
import Link from "next/link";

export default function HeroSection() {
	return (
		<section className="bg-primary/30 flex min-h-dvh w-full snap-start items-center pb-12">
			<div className="mx-auto max-w-7xl px-4 md:px-6">
				<div className="grid gap-6 lg:grid-cols-[1fr_400px] lg:gap-12 xl:grid-cols-[1fr_600px]">
					<Image
						src="/hero.webp?height=550&width=550"
						width={550}
						height={550}
						alt="Stacks Wars Lobby"
						className="mx-auto aspect-square overflow-hidden rounded-xl object-cover object-center sm:w-full lg:order-last"
					/>
					<div className="flex flex-col justify-center space-y-4">
						<div className="space-y-2">
							<h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl/none">
								Try Out Our First Featured Game!
							</h1>
							<p className="text-muted-foreground max-w-[600px] md:text-xl">
								Experience the thrill of Stacks Wars with our
								first game. Dive in, test your skills, and claim
								victory!
							</p>
						</div>
						<div className="flex flex-col gap-2 min-[400px]:flex-row">
							<Link href="/lobby">
								<Button size="lg" className="gap-1.5">
									Play Now
								</Button>
							</Link>
						</div>
					</div>
				</div>
			</div>
		</section>
	);
}
