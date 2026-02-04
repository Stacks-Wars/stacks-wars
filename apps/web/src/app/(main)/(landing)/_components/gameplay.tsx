import Image from "next/image";

export default function Gameplay() {
	return (
		<div className="w-full rounded-3xl lg:rounded-[48px] p-4 h-[405px] relative shadow-[0px_0px_34.07px_-18.09px_rgba(255,_255,_255,_0.25)_inset] [background:linear-gradient(150.88deg,_#2c61b8,_#000_65.11%)] border-solid border border-border-secondary overflow-hidden">
			<div className="rounded-[7.82px] lg:rounded-2xl bg-[#ffffff08] border-border-secondary border-solid border flex items-center justify-center py-4 px-4 shrink-0">
				<b className="max-w-[240.5px] shrink-0 text-center text-[19.6px]">
					<span>Engage in intense gameplay</span>
					<span className="text-xl"> and</span>
					<span> battle other warriors for dominance.</span>
				</b>
			</div>
			<div className="absolute bottom-0 right-0">
				<div className="relative">
					<Image
						className="object-cover"
						src={"/images/xbox-pad.png"}
						width={369.1}
						height={364.1}
						alt=""
					/>
					<Image
						className="absolute top-[20.83px] right-16 lg:w-13.25 lg:h-15.75 object-contain"
						width={25.9}
						src={"/images/ruby1.png"}
						height={30.9}
						alt=""
					/>
					<Image
						className="absolute bottom-12 left-[31.33px] w-[25.9px] h-[30.9px] object-contain"
						src={"/images/ruby2.png"}
						width={25.9}
						height={30.9}
						alt=""
					/>
				</div>
			</div>
		</div>
	);
}
