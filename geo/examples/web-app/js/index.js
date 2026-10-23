// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
import("../pkg").then((m) => {
  console.log("wasm is imported");
  const canvas = document.getElementById("cluster");
  const radius = 7;

  const ctx = canvas.getContext("2d");

  let cluster = m.compute_clusters();
  console.log(cluster);

  let clusterColor = ["blue", "green"];
  for (const [index, c] of cluster.entries()) {
    for (const p of c) {
      // Begin a new path and draw the circle
      ctx.beginPath();
      ctx.arc(p[0], p[1], radius, 0, 2 * Math.PI, false);

      // Set fill and stroke styles
      ctx.fillStyle = clusterColor[index];
      ctx.fill();
      ctx.lineWidth = 3;
      ctx.strokeStyle = "#000066";
      ctx.stroke();
    }
  }
});
