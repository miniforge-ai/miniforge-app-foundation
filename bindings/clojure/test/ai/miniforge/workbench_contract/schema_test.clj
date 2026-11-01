;; Title: Miniforge App Foundation
;; Subtitle: Clojure binding tests — the golden fixtures validate against the mirror
;; Author: Christopher Lester
;; Copyright 2025-2026 Christopher Lester (christopher@miniforge.ai)
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0
;;
;; Unless required by applicable law or agreed to in writing, software
;; distributed under the License is distributed on an "AS IS" BASIS,
;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;; See the License for the specific language governing permissions and
;; limitations under the License.

(ns ai.miniforge.workbench-contract.schema-test
  "The golden fixtures are the cross-language contract test. Every
   registry and snapshot under fixtures/<tenant>/ round-trips through the
   Rust types, so each must also validate against this Malli mirror."
  (:require
   [ai.miniforge.workbench-contract.schema :as sut]
   [cheshire.core :as json]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [malli.core :as m]
   [malli.error :as me]))

(def fixtures-dir
  "The repository's fixtures, relative to bindings/clojure (the test task's
   working directory)."
  (io/file "../../fixtures"))

(def parity-cases-file
  "Edge cases the Rust suite also runs (crates/workbench-contract/tests/parity.rs)."
  (io/file "../parity_cases.json"))

(def case-schemas
  "The mirror schema each parity case type validates against."
  {"registry" sut/StateVarRegistry
   "snapshot" sut/WorkbenchSnapshot
   "view_plugin" sut/ViewPlugin})

(defn- tenant-dirs
  []
  (->> (.listFiles fixtures-dir)
       (filter #(.isDirectory %))
       (sort-by #(.getName %))))

(defn- read-json
  [file]
  (json/parse-string (slurp file) true))

(defn- explain
  [schema value]
  (pr-str (me/humanize (m/explain schema value))))

(defn- pointer-path
  "The get-in path in `doc` for a JSON pointer: numeric segments index
   vectors, other segments are keyword map keys."
  [doc pointer]
  (:path (reduce (fn [{:keys [node path]} segment]
                   (let [k (if (sequential? node) (parse-long segment) (keyword segment))]
                     {:node (get node k) :path (conj path k)}))
                 {:node doc :path []}
                 (rest (str/split pointer #"/")))))

(defn- case-document
  "The case's inline `document`, or its base fixture with the value at
   `pointer` set (when the case has `set`) or removed."
  [{:keys [document base type pointer] :as parity-case}]
  (if document
    document
    (let [doc (read-json (io/file fixtures-dir base (str type ".json")))
          path (pointer-path doc pointer)]
      (cond
        (contains? parity-case :set) (assoc-in doc path (:set parity-case))
        ;; update-in with an empty path would add a nil key, not dissoc.
        (= 1 (count path)) (dissoc doc (peek path))
        :else (update-in doc (pop path) dissoc (peek path))))))

(deftest every-golden-fixture-validates-test
  (is (seq (tenant-dirs)) "no fixtures found; run from bindings/clojure")
  (doseq [dir (tenant-dirs)]
    (testing (.getName dir)
      (let [registry (read-json (io/file dir "registry.json"))
            snapshot (read-json (io/file dir "snapshot.json"))]
        (is (m/validate sut/StateVarRegistry registry)
            (explain sut/StateVarRegistry registry))
        (is (m/validate sut/WorkbenchSnapshot snapshot)
            (explain sut/WorkbenchSnapshot snapshot))))))

(deftest mirror-rejects-off-contract-snapshots-test
  (let [snapshot (read-json (io/file fixtures-dir "miniforge" "snapshot.json"))]
    (testing "a status outside the StateStatus enum"
      (is (not (m/validate sut/WorkbenchSnapshot
                           (assoc-in snapshot [:evaluations 0 :status] "green")))))
    (testing "a missing signature"
      (is (not (m/validate sut/WorkbenchSnapshot (dissoc snapshot :signature)))))))

(deftest mirror-matches-the-rust-parity-table-test
  (let [cases (:cases (read-json parity-cases-file))]
    (is (seq cases) "no parity cases found; run from bindings/clojure")
    (doseq [{:keys [name type valid] :as parity-case} cases]
      (testing name
        (let [schema (get case-schemas type)
              doc (case-document parity-case)]
          (is (some? schema) (str "unknown case type " type))
          (is (= valid (m/validate schema doc))
              (explain schema doc)))))))
