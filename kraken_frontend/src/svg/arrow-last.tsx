import React from "react";
import "../index.css";

export default function ArrowLastIcon(params: any) {
    return (
        <div className={"pagination-icon"} {...params}>
            <svg
                className="neon"
                fill="#000000"
                width="800px"
                height="800px"
                viewBox="0 0 1024 1024"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M562.19 511.799l-144.6 145.718-36.15-36.43L489.89 511.8l-108.45-109.29 36.15-36.429zM691.2 657.92H640V366.08h51.2z" />
            </svg>
        </div>
    );
}
