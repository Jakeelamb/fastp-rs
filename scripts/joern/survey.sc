// fastp-rs: non-interactive Joern survey of an existing CPG (from joern-parse).
// Run: joern --script scripts/joern/survey.sc --param cpgPath=/abs/path/to/fastp-cpg

import io.shiftleft.semanticcpg.language._

@main def main(cpgPath: String): Unit = {
  importCpg(cpgPath)
  println("=== distinct_method_name_count ===")
  println(cpg.method.name.l.distinct.size)
  println("=== distinct_typeDecl_name_count ===")
  println(cpg.typeDecl.name.l.distinct.size)
  println("=== typeDecl_names_sorted_sample_100 ===")
  cpg.typeDecl.name.l.distinct.sorted.take(100).foreach(println)
  println("=== method_names_sorted_sample_100 ===")
  cpg.method.name.l.distinct.sorted.take(100).foreach(println)
  println("=== done ===")
}
